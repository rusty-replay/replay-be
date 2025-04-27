use actix_web::{delete, get, post, put, web, HttpResponse};
use sea_orm::{Set, ActiveModelTrait, EntityTrait, QueryFilter, ColumnTrait, DatabaseConnection};
use sea_query::Condition;
use crate::entity::project_member::{Entity as ProjectMemberEntity, ActiveModel as ProjectMemberActiveModel};
use crate::entity::project::{Entity as ProjectEntity, ActiveModel as ProjectActiveModel};
use crate::entity::{project_member, user};
use crate::model::global_error::{AppError, ErrorCode};
use crate::model::project::{ProjectCreateRequest, ProjectDetailResponse, ProjectMemberResponse, ProjectResponse, ProjectUpdateRequest};

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
        updated_at: Set(now.into()),
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
        .filter_map(|(_, projects)| projects.first().cloned().map(ProjectResponse::from))
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

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
            joined_at: member.joined_at,
        }))
        .collect();

    Ok(HttpResponse::Ok().json(ProjectDetailResponse {
        project: ProjectResponse::from(project),
        members: member_responses,
    }))
}

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

    let mut project_model: ProjectActiveModel = project.into();

    if let Some(name) = &body.name {
        project_model.name = Set(name.clone());
    }
    if let Some(description) = &body.description {
        project_model.description = Set(Some(description.clone()));
    }
    project_model.updated_at = Set(chrono::Utc::now().into());

    let updated_project = project_model.update(db.get_ref()).await?;

    Ok(HttpResponse::Ok().json(ProjectResponse::from(updated_project)))
}

#[delete("/projects/{id}")]
pub async fn delete_project(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    auth_user: web::ReqData<i32>,
) -> Result<HttpResponse, AppError> {
    let project_id = path.into_inner();
    let user_id = *auth_user;

    check_project_owner(db.get_ref(), project_id, user_id).await?;

    ProjectEntity::delete_by_id(project_id)
        .exec(db.get_ref())
        .await?;

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