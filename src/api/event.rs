use std::env;
use actix_web::{get, post, put, web, HttpResponse};
use chrono::Utc;
use sea_orm::{EntityTrait, Set, ActiveModelTrait, QueryOrder, DatabaseConnection, QueryFilter, Condition, ColumnTrait, PaginatorTrait, DbErr, QuerySelect, QueryTrait};
use crate::entity::event::{self, ActiveModel as EventActiveModel, Entity as EventEntity};
use crate::entity::issue::{ActiveModel as IssueActiveModel, Entity as IssueEntity};
use crate::entity::project::{Entity as ProjectEntity};
use crate::entity::project_member::{self, Entity as ProjectMemberEntity};
use crate::model::event::{BatchEventReportRequest, BatchEventReportResponse, EventAssignee, EventPriority, EventQuery, EventReportListResponse, EventReportRequest, EventReportResponse, PaginatedResponse};
use sha2::{Sha256, Digest};
use crate::util::slack::send_slack_alert;
use crate::entity::{issue, project};
use crate::model::global_error::{AppError, ErrorCode};
use std::sync::LazyLock;
use crate::api::project::check_project_member;

async fn find_project_by_api_key(db: &DatabaseConnection, api_key: &str) -> Result<i32, AppError> {
    let project = ProjectEntity::find()
        .filter(project::Column::ApiKey.eq(api_key))
        .one(db)
        .await?
        .ok_or_else(|| AppError::bad_request(ErrorCode::InvalidApiKey))?;

    Ok(project.id)
}

async fn create_or_update_issue(db: &DatabaseConnection, project_id: i32, group_hash: &str, message: &str) -> Result<i32, AppError> {
    let now = Utc::now();

    let existing_issue = IssueEntity::find()
        .filter(
            issue::Column::ProjectId.eq(project_id)
                .and(issue::Column::GroupHash.eq(group_hash))
        )
        .one(db)
        .await?;

    if let Some(issue) = existing_issue {
        let mut issue_model: issue::ActiveModel = issue.clone().into();
        issue_model.count = Set(issue.count + 1);
        issue_model.last_seen = Set(now.into());
        issue_model.updated_at = Set(now.into());

        let updated_issue = issue_model.update(db).await?;

        Ok(updated_issue.id)
    } else {
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

        let inserted_issue = new_issue.insert(db).await?;

        Ok(inserted_issue.id)
    }
}

#[utoipa::path(
    post,
    path = "/batch-events",
    summary = "이벤트 batch report",
    request_body = BatchEventReportRequest,
    responses(
        (status = 200, description = "이벤트 전송 성공", body = BatchEventReportResponse),
    ),
)]
#[post("/batch-events")]
pub async fn report_batch_events(
    body: web::Json<BatchEventReportRequest>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    println!("Batch event report: {:?}", body);

    let mut success_count = 0;
    let mut events = Vec::new();
    let mut project_id_opt: Option<i32> = None;

    println!("Processing {} events", body.events.len());

    for (index, event) in body.events.iter().enumerate() {
        match process_event(db.get_ref(), event).await {
            Ok(pid) => {
                success_count += 1;
                project_id_opt = Some(pid);
            },
            Err(e) => events.push(format!("이벤트 #{} 처리 중 오류: {}", index, e)),
        }
    }

    if let Some(project_id) = project_id_opt {
        let count = EventEntity::find()
            .filter(event::Column::ProjectId.eq(project_id))
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

    Ok(HttpResponse::Ok().json(BatchEventReportResponse {
        processed: body.events.len(),
        success: success_count,
        events,
    }))
}

async fn process_event(
    db: &DatabaseConnection,
    event: &EventReportRequest,
) -> Result<i32, AppError> {
    let project_id = find_project_by_api_key(db, &event.api_key).await?;
    let group_hash = calculate_group_hash(&event.message, &event.stacktrace);
    let issue_id = create_or_update_issue(db, project_id, &group_hash, &event.message).await?;

    let new_log = EventActiveModel::from_error_event(event, project_id, issue_id, group_hash);
    match new_log.insert(db).await {
        Ok(_) => Ok(project_id),
        Err(DbErr::Exec(error)) => {
            println!("DB Exec Error: {}", error);
            // 예: insert 실행 실패했을 때 (SQL syntax error, constraint error 등)
            Err(AppError::internal_error(ErrorCode::InternalError))
        }
        Err(DbErr::Query(error)) => {
            println!("DB Exec Error: {}", error);
            // 예: query 실패했을 때
            Err(AppError::internal_error(ErrorCode::InternalError))
        }
        Err(other) => {
            println!("DB Exec Error: {}", other);
            // 다른 모든 에러
            Err(AppError::internal_error(ErrorCode::InternalError))
        }
    }.expect("TODO: panic message");

    // let _ = new_log.insert(db).await?;

    Ok(project_id)
}

#[utoipa::path(
    post,
    path = "/events",
    summary = "이벤트 단일 report",
    request_body = EventReportRequest,
    responses(
        (status = 201, description = "이벤트 전송 성공", body = EventReportListResponse),
    ),
)]
#[post("/events")]
pub async fn report_event(
    body: web::Json<EventReportRequest>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    let project_id = find_project_by_api_key(db.get_ref(), &body.api_key).await?;
    let group_hash = calculate_group_hash(&body.message, &body.stacktrace);
    let issue_id = create_or_update_issue(db.get_ref(), project_id, &group_hash, &body.message).await?;
    let new_log = EventActiveModel::from_error_event(
        &body,
        project_id,
        issue_id,
        group_hash.clone(),
    );

    let inserted = new_log.insert(db.get_ref()).await?;

    Ok(HttpResponse::Created().json(EventReportListResponse::from(inserted)))
}

#[utoipa::path(
    get,
    path = "/api/projects/{project_id}/events",
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("search" = Option<String>, Query, description = "검색어"),
        ("page" = Option<i32>, Query, description = "페이지 번호"),
        ("page_size" = Option<i32>, Query, description = "페이지 크기"),
        ("start_date" = Option<String>, Query, description = "시작일 (ISO8601)"),
        ("end_date" = Option<String>, Query, description = "종료일 (ISO8601)")
    ),
    responses(
        (status = 200, description = "이벤트 목록 조회 성공", body = Vec<EventReportListResponse>),
    ),
)]
#[get("/projects/{project_id}/events")]
pub async fn list_project_events(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    query: web::Query<EventQuery>,
    auth_user: web::ReqData<i32>,
) -> Result<HttpResponse, AppError> {
    let project_id = path.into_inner();
    let user_id = auth_user.into_inner();

    check_project_member(db.get_ref(), project_id, user_id).await?;

    let EventQuery { search, page, page_size, start_date, end_date } = query.into_inner();

    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let total_elements = EventEntity::find()
        .filter(event::Column::ProjectId.eq(project_id))
        .count(db.get_ref())
        .await?;

    let mut query = EventEntity::find()
        .filter(event::Column::ProjectId.eq(project_id));

    if let Some(search_term) = search {
        let pattern = format!("%{}%", search_term);
        query = query.filter(
            Condition::any()
                .add(event::Column::Message.like(&pattern))
                .add(event::Column::Stacktrace.like(&pattern))
                .add(event::Column::AppVersion.like(&pattern))
        );
    }

    if let Some(start) = start_date {
        query = query.filter(event::Column::Timestamp.gte(start));
    }
    if let Some(end) = end_date {
        query = query.filter(event::Column::Timestamp.lte(end));
    }

    let filtered_elements = query.clone().count(db.get_ref()).await?;

    let logs = query
        .order_by_desc(event::Column::CreatedAt)
        .order_by_desc(event::Column::Id)
        .offset(Some(offset as u64))
        .limit(Some(page_size as u64))
        .all(db.get_ref())
        .await?;

    let response = PaginatedResponse {
        content: logs
            .into_iter()
            .map(EventReportListResponse::from)
            .collect::<Vec<_>>(),
        page,
        page_size,
        total_elements,
        filtered_elements,
        total_pages: ((filtered_elements as f64) / (page_size as f64)).ceil() as u32,
        has_next: (offset + page_size) < (filtered_elements as u32),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/api/projects/{project_id}/events/{id}",
    summary = "프로젝트 이벤트 상세 조회",
    responses(
        (status = 200, description = "이벤트 상세 조회 성공", body = EventReportResponse),
    ),
)]
#[get("/projects/{project_id}/events/{id}")]
pub async fn get_project_events(
    db: web::Data<DatabaseConnection>,
    path: web::Path<(i32, i32)>,
    auth_user: web::ReqData<i32>,
) -> Result<HttpResponse, AppError> {
    let (project_id, event_id) = path.into_inner();
    let user_id = auth_user.into_inner();

    let is_member = ProjectMemberEntity::find()
        .filter(
            Condition::all()
                .add(project_member::Column::ProjectId.eq(project_id))
                .add(project_member::Column::UserId.eq(user_id))
        )
        .one(db.get_ref())
        .await?;

    if is_member.is_none() {
        return Err(AppError::forbidden(ErrorCode::NotEnoughPermission));
    }

    let log = EventEntity::find()
        .filter(
            Condition::all()
                .add(event::Column::Id.eq(event_id))
                .add(event::Column::ProjectId.eq(project_id))
        )
        .one(db.get_ref())
        .await?
        .ok_or_else(|| AppError::not_found(ErrorCode::ErrorLogNotFound))?;


    Ok(HttpResponse::Ok().json(EventReportResponse::from(log)))
}

#[utoipa::path(
    put,
    path = "/api/projects/{project_id}/events/{id}/priority",
    summary = "이벤트 우선순위 설정",
    request_body = EventPriority,
    responses(
        (status = 200, description = "이벤트 우선순위 설정 성공", body = EventReportListResponse),
    ),
)]
#[put("/projects/{project_id}/events/{id}/priority")]
pub async fn set_priority(
    db: web::Data<DatabaseConnection>,
    path: web::Path<(i32, i32)>,
    auth_user: web::ReqData<i32>,
    body: web::Json<EventPriority>,
) -> Result<HttpResponse, AppError> {
    let (project_id, event_id) = path.into_inner();
    let user_id = auth_user.into_inner();
    let priority = &body.priority;

    check_project_member(db.get_ref(), project_id, user_id).await?;

    let event = find_event(db.get_ref(), project_id, event_id).await?;
    let mut active_model: EventActiveModel = event.into();
    active_model.priority = Set(Some(*priority));
    active_model.updated_at = Set(Some(Utc::now()));

    let updated = active_model.update(db.get_ref()).await?;

    Ok(HttpResponse::Ok().json(EventReportListResponse::from(updated)))
}

#[utoipa::path(
    put,
    path = "/api/projects/{project_id}/events/{id}/assignee",
    summary = "이벤트 담당자 설정",
    request_body = EventAssignee,
    responses(
        (status = 200, description = "담당자 설정 성공", body = EventReportListResponse),
        (status = 400, description = "잘못된 사용자"),
        (status = 403, description = "접근 권한 없음"),
        (status = 404, description = "이벤트 또는 프로젝트 없음"),
    ),
)]
#[put("/projects/{project_id}/events/{id}/assignee")]
pub async fn set_assignee(
    db: web::Data<DatabaseConnection>,
    path: web::Path<(i32, i32)>,
    auth_user: web::ReqData<i32>,
    body: web::Json<EventAssignee>,
) -> Result<HttpResponse, AppError> {
    let (project_id, event_id) = path.into_inner();
    let user_id = auth_user.into_inner();
    let assigned_to = body.assigned_to;

    check_project_member(db.get_ref(), project_id, user_id).await?;

    if let Some(target_user_id) = assigned_to {
        let is_member = ProjectMemberEntity::find()
            .filter(
                project_member::Column::ProjectId.eq(project_id)
                    .and(project_member::Column::UserId.eq(target_user_id))
            )
            .one(db.get_ref())
            .await?;

        if is_member.is_none() {
            return Err(AppError::bad_request(ErrorCode::InvalidAssignee));
        }
    }

    let event = find_event(db.get_ref(), project_id, event_id).await?;
    let mut active_model: EventActiveModel = event.into();
    active_model.assigned_to = Set(assigned_to);
    active_model.updated_at = Set(Some(Utc::now()));

    let updated = active_model.update(db.get_ref()).await?;

    Ok(HttpResponse::Ok().json(EventReportListResponse::from(updated)))
}

static SLACK_WEBHOOK_URL: LazyLock<String> = LazyLock::new(|| {
    env::var("SLACK_WEBHOOK_URL").expect("SLACK_WEBHOOK_URL 환경 변수가 설정되어야 합니다.")
});
const ERROR_THRESHOLD: usize = 1;

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

pub async fn find_event(
    db: &DatabaseConnection,
    project_id: i32,
    event_id: i32,
) -> Result<crate::entity::event::Model, AppError> {
    let event = EventEntity::find()
        .filter(
            Condition::all()
                .add(event::Column::Id.eq(event_id))
                .add(event::Column::ProjectId.eq(project_id))
        )
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found(ErrorCode::ErrorLogNotFound))?;

    Ok(event)
}