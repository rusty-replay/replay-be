use actix_web::{delete, get, post, put, web, HttpResponse};
use chrono::Utc;
use sea_orm::{Set, ActiveModelTrait, EntityTrait, QueryFilter, ColumnTrait, DatabaseConnection};
use sea_query::Condition;
use crate::entity::project_member::{Entity as ProjectMemberEntity, ActiveModel as ProjectMemberActiveModel};
use crate::entity::project::{Entity as ProjectEntity, ActiveModel as ProjectActiveModel};
use crate::entity::{project_member, user};
use crate::model::global_error::{AppError, ErrorCode};
use crate::model::project::{ProjectCreateRequest, ProjectDetailResponse, ProjectMemberResponse, ProjectResponse, ProjectUpdateRequest};

#[utoipa::path(
    post,
    path = "/api/projects",
    summary = "프로젝트 생성",
    request_body = ProjectCreateRequest,
    responses(
        (status = 201, description = "프로젝트 생성 성공", body = ProjectResponse),
    ),
)]
#[post("/projects")]
pub async fn create_project(
    body: web::Json<ProjectCreateRequest>,
    db: web::Data<DatabaseConnection>,
    auth_user: web::ReqData<i32>,
) -> Result<HttpResponse, AppError> {
    let api_key = format!("proj_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    let now = chrono::Utc::now();

    let new_project = ProjectActiveModel {
        name: Set(body.name.clone()),
        api_key: Set(api_key),
        description: Set(body.description.clone()),
        created_at: Set(now.into()),
        updated_at: Set(None),
        ..Default::default()
    };

    let inserted_project = new_project.insert(db.get_ref()).await?;

    let member = ProjectMemberActiveModel {
        user_id: Set(*auth_user),
        project_id: Set(inserted_project.id),
        role: Set("owner".to_string()),
        joined_at: Set(now.into()),
    };

    member.insert(db.get_ref()).await?;

    Ok(HttpResponse::Created().json(ProjectResponse::from(inserted_project)))
}


#[utoipa::path(
    get,
    path = "/api/projects",
    summary = "프로젝트 목록 조회",
    responses(
        (status = 200, description = "프로젝트 목록 조회 성공", body = Vec<ProjectResponse>),
    ),
)]
#[get("/projects")]
pub async fn list_user_projects(
    db: web::Data<DatabaseConnection>,
    auth_user: web::ReqData<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id = *auth_user;

    let projects = ProjectMemberEntity::find()
        .filter(project_member::Column::UserId.eq(user_id))
        .find_with_related(ProjectEntity)
        .all(db.get_ref())
        .await?;

    let response: Vec<ProjectResponse> = projects
        .into_iter()
        .filter_map(|(_, projects)| {
            projects
                .iter()
                .find(|p| p.deleted_at.is_none())
                .cloned()
                .map(ProjectResponse::from)
            // projects.first().cloned().map(ProjectResponse::from)
        })
        .collect();

    println!("projects list: {:?}", response);

    Ok(HttpResponse::Ok().json(response))
}


#[utoipa::path(
    get,
    path = "/api/projects/{id}",
    summary = "프로젝트 상세 정보 조회",
    params(
        ("id", description = "프로젝트 ID", example = 1),
    ),
    responses(
        (status = 200, description = "프로젝트 상세 조회 성공", body = ProjectDetailResponse),
    ),
)]
#[get("/projects/{id}")]
pub async fn get_project(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    auth_user: web::ReqData<i32>,
) -> Result<HttpResponse, AppError> {
    let project_id = path.into_inner();
    let user_id = *auth_user;

    ProjectEntity::find_by_id(project_id)
        .one(db.get_ref())
        .await?
        .ok_or_else(|| AppError::not_found(ErrorCode::ProjectNotFound))?;

    check_project_member(db.get_ref(), project_id, user_id).await?;

    let project = ProjectEntity::find_by_id(project_id)
        .one(db.get_ref())
        .await?
        .filter(|p| p.deleted_at.is_none())
        .ok_or_else(|| AppError::not_found(ErrorCode::ProjectNotFound))?;

    let members = ProjectMemberEntity::find()
        .filter(project_member::Column::ProjectId.eq(project_id))
        .find_with_related(user::Entity)
        .all(db.get_ref())
        .await?;

    let member_responses: Vec<ProjectMemberResponse> = members
        .into_iter()
        .filter_map(|(member, users)| users.first().map(|u| ProjectMemberResponse {
            user_id: u.id,
            username: u.username.clone(),
            email: u.email.clone(),
            role: member.role.clone(),
            joined_at: member.joined_at.into(),
        }))
        .collect();

    Ok(HttpResponse::Ok().json(ProjectDetailResponse {
        project: ProjectResponse::from(project),
        members: member_responses,
    }))
}

#[utoipa::path(
    put,
    path = "/api/projects/{id}",
    summary = "프로젝트 업데이트",
    request_body = ProjectUpdateRequest,
    responses(
        (status = 200, description = "프로젝트 수정 성공", body = ProjectResponse),
    ),
)]
#[put("/projects/{id}")]
pub async fn update_project(
    path: web::Path<i32>,
    body: web::Json<ProjectUpdateRequest>,
    db: web::Data<DatabaseConnection>,
    auth_user: web::ReqData<i32>,
) -> Result<HttpResponse, AppError> {
    let project_id = path.into_inner();
    let user_id = *auth_user;

    check_project_member(db.get_ref(), project_id, user_id).await?;

    let project = ProjectEntity::find_by_id(project_id)
        .one(db.get_ref())
        .await?
        .ok_or_else(|| AppError::not_found(ErrorCode::ProjectNotFound))?;

    if project.deleted_at.is_some() {
        return Err(AppError::not_found(ErrorCode::ProjectNotFound));
    }

    let mut project_model: ProjectActiveModel = project.into();

    if let Some(name) = &body.name {
        project_model.name = Set(name.clone());
    }
    if let Some(description) = &body.description {
        project_model.description = Set(Some(description.clone()));
    }
    project_model.updated_at = Set(Some(Utc::now()));

    let updated_project = project_model.update(db.get_ref()).await?;

    Ok(HttpResponse::Ok().json(ProjectResponse::from(updated_project)))
}

#[utoipa::path(
    delete,
    path = "/api/projects/{id}",
    summary = "프로젝트 삭제",
    responses(
        (status = 204, description = "프로젝트 삭제 성공"),
    ),
)]
#[delete("/projects/{id}")]
pub async fn delete_project(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    auth_user: web::ReqData<i32>,
) -> Result<HttpResponse, AppError> {
    let project_id = path.into_inner();
    let user_id = *auth_user;

    check_project_owner(db.get_ref(), project_id, user_id).await?;

    let project = ProjectEntity::find_by_id(project_id)
        .one(db.get_ref())
        .await?
        .ok_or_else(|| AppError::not_found(ErrorCode::ProjectNotFound))?;

    if project.deleted_at.is_some() {
        return Err(AppError::not_found(ErrorCode::ProjectNotFound));
    }

    let mut project_model: ProjectActiveModel = project.into();
    project_model.soft_delete(user_id.into());
    project_model.update(db.get_ref()).await?;

    Ok(HttpResponse::NoContent().finish())
}

// TODO 5. 프로젝트 삭제
// TODO  6. 프로젝트에 유저 초대
// TODO 7. 프로젝트에서 멤버 제거


pub async fn check_project_member(
    db: &DatabaseConnection,
    project_id: i32,
    user_id: i32,
) -> Result<(), AppError> {
    let is_member = ProjectMemberEntity::find()
        .filter(
            Condition::all()
                .add(project_member::Column::ProjectId.eq(project_id))
                .add(project_member::Column::UserId.eq(user_id))
        )
        .one(db)
        .await?;

    if is_member.is_none() {
        return Err(AppError::forbidden(ErrorCode::NotEnoughPermission));
    }

    Ok(())
}

pub async fn check_project_owner(
    db: &DatabaseConnection,
    project_id: i32,
    user_id: i32,
) -> Result<(), AppError> {
    let is_owner = ProjectMemberEntity::find()
        .filter(
            Condition::all()
                .add(project_member::Column::ProjectId.eq(project_id))
                .add(project_member::Column::UserId.eq(user_id))
                .add(project_member::Column::Role.eq("owner"))
        )
        .one(db)
        .await?;

    if is_owner.is_none() {
        return Err(AppError::forbidden(ErrorCode::NotEnoughPermission));
    }

    Ok(())
}