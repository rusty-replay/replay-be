use std::env;
use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::Utc;
use once_cell::sync::Lazy;
use sea_orm::{EntityTrait, Set, ActiveModelTrait, QueryOrder, DatabaseConnection, QueryFilter, Condition, ColumnTrait, JoinType, PaginatorTrait};
use crate::entity::error_log::{self, ActiveModel as ErrorLogActiveModel, Entity as ErrorEntity};
use crate::entity::issue::{ActiveModel as IssueActiveModel, Entity as IssueEntity};
use crate::entity::project::{Entity as ProjectEntity};
use crate::entity::project_member::{self, Entity as ProjectMemberEntity};
use crate::entity::user::{Entity as UserEntity};
use crate::entity::error_log::{Entity as ErrorLogEntity};
use crate::model::error::{BatchErrorReportRequest, BatchErrorReportResponse, ErrorReportListResponse, ErrorReportRequest, ErrorReportResponse};
use sha2::{Sha256, Digest};
use crate::util::slack::send_slack_alert;
use crate::entity::{issue, project};
use crate::model::global_error::{AppError, ErrorCode};

static SLACK_WEBHOOK_URL: Lazy<String> = Lazy::new(|| {
    env::var("SLACK_WEBHOOK_URL").expect("SLACK_WEBHOOK_URL 환경 변수가 설정되어야 합니다.")
});
const ERROR_THRESHOLD: usize = 1;

#[utoipa::path(
    get,
    path = "/health-check",
    responses(
        (status = 200, description = "서버가 정상 동작 중", body = String)
    )
)]
#[get("/health-check")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

fn calculate_group_hash(message: &str, stack: &str) -> String {
    // 메시지에서 변수 부분 정규화 (숫자, ID 등 제거)
    let normalized_message = message
        .replace(|c: char| c.is_numeric(), "0")
        .replace(|c: char| c.is_ascii_hexdigit() && !c.is_numeric(), "X");

    // 스택트레이스에서 중요 부분만 추출 (파일 경로, 라인 번호 제외)
    let stack_lines: Vec<&str> = stack.lines().collect();
    let mut important_stack = String::new();

    // stack trace 처음 3줄만 사용
    for i in 0..std::cmp::min(3, stack_lines.len()) {
        if let Some(func_pos) = stack_lines[i].find("at ") {
            if let Some(file_pos) = stack_lines[i][func_pos..].find(" (") {
                important_stack.push_str(&stack_lines[i][func_pos..func_pos+file_pos]);
                important_stack.push('\n');
            } else {
                important_stack.push_str(stack_lines[i]);
                important_stack.push('\n');
            }
        }
    }

    let mut hasher = Sha256::new();
    hasher.update(normalized_message);
    hasher.update(important_stack);
    let result = hasher.finalize();
    format!("{:x}", result)
}

async fn find_project_by_api_key(db: &DatabaseConnection, api_key: &str) -> Result<i32, AppError> {
    let project = ProjectEntity::find()
        .filter(project::Column::ApiKey.eq(api_key))
        .one(db)
        .await
        .map_err(|_| AppError::new(ErrorCode::DatabaseError))?
        .ok_or_else(|| AppError::new(ErrorCode::InvalidApiKey))?;

    Ok(project.id)
}

async fn create_or_update_issue(db: &DatabaseConnection, project_id: i32, group_hash: &str, message: &str) -> Result<i32, AppError> {
    let now = Utc::now();

    // 기존 이슈 찾기
    let existing_issue = IssueEntity::find()
        .filter(
            issue::Column::ProjectId.eq(project_id)
                .and(issue::Column::GroupHash.eq(group_hash))
        )
        .one(db)
        .await
        .map_err(|_| AppError::new(ErrorCode::DatabaseError))?;

    if let Some(issue) = existing_issue {
        // 기존 이슈 업데이트
        let mut issue_model: issue::ActiveModel = issue.clone().into();
        issue_model.count = Set(issue.count + 1);
        issue_model.last_seen = Set(now.into());
        issue_model.updated_at = Set(now.into());

        let updated_issue = issue_model.update(db).await
            .map_err(|_| AppError::new(ErrorCode::DatabaseError))?;

        Ok(updated_issue.id)
    } else {
        // 새 이슈 생성
        let title = if message.len() > 100 {
            format!("{}...", &message[..97])
        } else {
            message.to_string()
        };

        let new_issue = IssueActiveModel {
            title: Set(title),
            group_hash: Set(group_hash.to_string()),
            status: Set("open".to_string()),
            first_seen: Set(now.into()),
            last_seen: Set(now.into()),
            count: Set(1),
            project_id: Set(project_id),
            assigned_to: Set(None),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            ..Default::default()
        };

        let inserted_issue = new_issue.insert(db).await
            .map_err(|_| AppError::new(ErrorCode::DatabaseError))?;

        Ok(inserted_issue.id)
    }
}

#[post("/batch-errors")]
pub async fn report_batch_errors(
    body: web::Json<BatchErrorReportRequest>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    println!("Batch error report: {:?}", body);

    let mut success_count = 0;
    let mut errors = Vec::new();
    let mut project_id_opt: Option<i32> = None;

    println!("Processing {} events", body.events.len());

    for (index, event) in body.events.iter().enumerate() {
        match process_event(db.get_ref(), event).await {
            Ok(pid) => {
                success_count += 1;
                project_id_opt = Some(pid);
            },
            Err(e) => errors.push(format!("이벤트 #{} 처리 중 오류: {}", index, e)),
        }
    }

    if let Some(project_id) = project_id_opt {
        let count = ErrorLogEntity::find()
            .filter(error_log::Column::ProjectId.eq(project_id))
            .count(db.get_ref())
            .await
            .unwrap_or(0);

        if count >= ERROR_THRESHOLD as u64 {
            let _ = send_slack_alert(
                &SLACK_WEBHOOK_URL,
                &format!("🚨 Project {} 에 에러가 {}개 이상 발생했습니다.", project_id, count),
            ).await;
        }
    }

    Ok(HttpResponse::Ok().json(BatchErrorReportResponse {
        processed: body.events.len(),
        success: success_count,
        errors,
    }))
}

async fn process_event(
    db: &DatabaseConnection,
    event: &ErrorReportRequest,
) -> Result<i32, AppError> {
    let project_id = find_project_by_api_key(db, &event.api_key).await?;
    let group_hash = calculate_group_hash(&event.message, &event.stacktrace);
    let issue_id = create_or_update_issue(db, project_id, &group_hash, &event.message).await?;

    let new_log = ErrorLogActiveModel::from_error_event(event, project_id, issue_id, group_hash);

    let _ = new_log.insert(db).await
        .map_err(|e| {
            log::error!("에러 로그 저장 중 오류 발생: {}", e);
            AppError::new(ErrorCode::DatabaseError)
        })?;

    Ok(project_id)
}

#[post("/errors")]
pub async fn report_error(
    body: web::Json<ErrorReportRequest>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    let project_id = find_project_by_api_key(db.get_ref(), &body.api_key).await?;
    let group_hash = calculate_group_hash(&body.message, &body.stacktrace);
    let issue_id = create_or_update_issue(db.get_ref(), project_id, &group_hash, &body.message).await?;
    let new_log = ErrorLogActiveModel::from_error_event(
        &body,
        project_id,
        issue_id,
        group_hash.clone(),
    );

    let inserted = new_log.insert(db.get_ref()).await
        .map_err(|e| {
            log::error!("에러 로그 저장 중 오류 발생: {}", e);
            AppError::new(ErrorCode::DatabaseError)
        })?;

    Ok(HttpResponse::Created().json(ErrorReportListResponse::from(inserted)))
}

#[get("/projects/{project_id}/errors")]
pub async fn list_project_errors(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    auth_user: web::ReqData<i32>,
) -> Result<HttpResponse, AppError> {
    let project_id = path.into_inner();
    let user_id = auth_user.into_inner();

    let is_member = ProjectMemberEntity::find()
        .filter(
            Condition::all()
                .add(project_member::Column::ProjectId.eq(project_id))
                .add(project_member::Column::UserId.eq(user_id))
        )
        .one(db.get_ref())
        .await
        .map_err(|err| {
            log::error!("프로젝트 멤버십 확인 중 오류 발생: {}", err);
            AppError::new(ErrorCode::DatabaseError)
        })?;

    if is_member.is_none() {
        return Err(AppError::new(ErrorCode::NotEnoughPermission));
    }

    let logs = ErrorLogEntity::find()
        .filter(error_log::Column::ProjectId.eq(project_id))
        .order_by_desc(error_log::Column::CreatedAt)
        // .limit(100)
        .all(db.get_ref())
        .await
        .map_err(|_| AppError::new(ErrorCode::DatabaseError))?;

    let response: Vec<ErrorReportListResponse> = logs
        .into_iter()
        .map(|l| ErrorReportListResponse::from(l))
        // .map(Into::into)
        // .map(|l| l.into())
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

#[get("/projects/{project_id}/errors/{id}")]
pub async fn get_project_error(
    db: web::Data<DatabaseConnection>,
    path: web::Path<(i32, i32)>,
    auth_user: web::ReqData<i32>,
) -> Result<HttpResponse, AppError> {
    let (project_id, error_id) = path.into_inner();
    let user_id = auth_user.into_inner();

    let is_member = ProjectMemberEntity::find()
        .filter(
            Condition::all()
                .add(project_member::Column::ProjectId.eq(project_id))
                .add(project_member::Column::UserId.eq(user_id))
        )
        .one(db.get_ref())
        .await
        .map_err(|err| {
            log::error!("프로젝트 멤버십 확인 중 오류 발생: {}", err);
            AppError::new(ErrorCode::DatabaseError)
        })?;

    if is_member.is_none() {
        return Err(AppError::new(ErrorCode::NotEnoughPermission));
    }

    let log = ErrorLogEntity::find()
        .filter(
            Condition::all()
                .add(error_log::Column::Id.eq(error_id))
                .add(error_log::Column::ProjectId.eq(project_id))
        )
        .one(db.get_ref())
        .await
        .map_err(|_| AppError::new(ErrorCode::DatabaseError))?
        .ok_or_else(|| AppError::new(ErrorCode::ErrorLogNotFound))?;


    Ok(HttpResponse::Ok().json(ErrorReportResponse::from(log)))
}
