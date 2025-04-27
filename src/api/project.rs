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

    let is_owner = ProjectMemberEntity::find()
        .filter(
            Condition::all()
                .add(project_member::Column::ProjectId.eq(project_id))
                .add(project_member::Column::UserId.eq(user_id))
                .add(project_member::Column::Role.eq("owner"))
        )
        .one(db.get_ref())
        .await?;

    if is_owner.is_none() {
        return Err(AppError::forbidden(ErrorCode::NotEnoughPermission));
    }

    ProjectEntity::delete_by_id(project_id)
        .exec(db.get_ref())
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

// #[put("/projects/{id}")]
// pub async fn update_project(
//     db: web::Data<sea_orm::DatabaseConnection>,
//     path: web::Path<i32>,
//     body: web::Json<ProjectUpdateRequest>,
//     auth_user: web::ReqData<i32>,
// ) -> impl Responder {
//     let project_id = path.into_inner();
//     let user_id = *auth_user.into_inner();
//
//     // 사용자가 프로젝트의 관리자인지 확인
//     let is_admin = ProjectMemberEntity::find()
//         .filter(
//             Condition::all()
//                 .add(project_member::Column::ProjectId.eq(project_id))
//                 .add(project_member::Column::UserId.eq(user_id))
//                 .add(project_member::Column::Role.is_in(vec!["owner", "admin"]))
//         )
//         .one(db.get_ref())
//         .await
//         .unwrap()
//         .is_some();
//
//     if !is_admin {
//         return HttpResponse::Forbidden().body("You don't have permission to update this project");
//     }
//
//     // 프로젝트 정보 조회
//     let project_result = ProjectEntity::find_by_id(project_id)
//         .one(db.get_ref())
//         .await
//         .unwrap();
//
//     if let Some(project) = project_result {
//         let mut project_model: ProjectActiveModel = project.into();
//
//         // 업데이트할 필드가 있는 경우에만 업데이트
//         if let Some(name) = &body.name {
//             project_model.name = Set(name.clone());
//         }
//
//         if let Some(description) = &body.description {
//             project_model.description = Set(Some(description.clone()));
//         }
//
//         project_model.updated_at = Set(chrono::Utc::now().into());
//
//         let updated_project = project_model.update(db.get_ref()).await.unwrap();
//
//         HttpResponse::Ok().json(ProjectResponse {
//             id: updated_project.id,
//             name: updated_project.name,
//             api_key: updated_project.api_key,
//             description: updated_project.description,
//             created_at: updated_project.created_at,
//             updated_at: updated_project.updated_at,
//         })
//     } else {
//         HttpResponse::NotFound().body("Project not found")
//     }
// }
//
// // 5. 프로젝트 삭제
// #[delete("/projects/{id}")]
// pub async fn delete_project(
//     db: web::Data<sea_orm::DatabaseConnection>,
//     path: web::Path<i32>,
//     auth_user: web::ReqData<i32>,
// ) -> impl Responder {
//     let project_id = path.into_inner();
//     let user_id = *auth_user.into_inner();
//
//     // 사용자가 프로젝트의 소유자인지 확인
//     let is_owner = ProjectMemberEntity::find()
//         .filter(
//             Condition::all()
//                 .add(project_member::Column::ProjectId.eq(project_id))
//                 .add(project_member::Column::UserId.eq(user_id))
//                 .add(project_member::Column::Role.eq("owner"))
//         )
//         .one(db.get_ref())
//         .await
//         .unwrap()
//         .is_some();
//
//     if !is_owner {
//         return HttpResponse::Forbidden().body("Only the project owner can delete the project");
//     }
//
//     // 프로젝트 삭제 (외래 키 제약 조건에 따라 관련 프로젝트 멤버도 삭제됨)
//     let _ = ProjectEntity::delete_by_id(project_id)
//         .exec(db.get_ref())
//         .await
//         .unwrap();
//
//     HttpResponse::NoContent().finish()
// }
//
// // 6. 프로젝트에 유저 초대
// #[post("/projects/{id}/members")]
// pub async fn invite_user(
//     db: web::Data<sea_orm::DatabaseConnection>,
//     path: web::Path<i32>,
//     body: web::Json<ProjectInviteRequest>,
//     auth_user: web::ReqData<i32>,
// ) -> impl Responder {
//     let project_id = path.into_inner();
//     let user_id = *auth_user.into_inner();
//
//     // 사용자가 프로젝트의 관리자인지 확인
//     let is_admin = ProjectMemberEntity::find()
//         .filter(
//             Condition::all()
//                 .add(project_member::Column::ProjectId.eq(project_id))
//                 .add(project_member::Column::UserId.eq(user_id))
//                 .add(project_member::Column::Role.is_in(vec!["owner", "admin"]))
//         )
//         .one(db.get_ref())
//         .await
//         .unwrap()
//         .is_some();
//
//     if !is_admin {
//         return HttpResponse::Forbidden().body("You don't have permission to invite users");
//     }
//
//     // 대상 사용자가 이미 프로젝트 멤버인지 확인
//     let already_member = ProjectMemberEntity::find()
//         .filter(
//             Condition::all()
//                 .add(project_member::Column::ProjectId.eq(project_id))
//                 .add(project_member::Column::UserId.eq(body.user_id))
//         )
//         .one(db.get_ref())
//         .await
//         .unwrap()
//         .is_some();
//
//     if already_member {
//         return HttpResponse::BadRequest().body("User is already a member of this project");
//     }
//
//     // 사용자가 존재하는지 확인
//     let target_user = user::Entity::find_by_id(body.user_id)
//         .one(db.get_ref())
//         .await
//         .unwrap();
//
//     if target_user.is_none() {
//         return HttpResponse::BadRequest().body("User not found");
//     }
//
//     // 역할 유효성 검증 (간단한 예시)
//     let valid_roles = vec!["admin", "member", "viewer"];
//     if !valid_roles.contains(&body.role.as_str()) {
//         return HttpResponse::BadRequest().body("Invalid role");
//     }
//
//     // 프로젝트 멤버 추가
//     let new_member = ProjectMemberActiveModel {
//         user_id: Set(body.user_id),
//         project_id: Set(project_id),
//         role: Set(body.role.clone()),
//         joined_at: Set(chrono::Utc::now().into()),
//     };
//
//     let _ = new_member.insert(db.get_ref()).await.unwrap();
//
//     // 사용자 정보와 함께 응답
//     let user_info = target_user.unwrap();
//
//     HttpResponse::Created().json(ProjectMemberResponse {
//         user_id: user_info.id,
//         username: user_info.username,
//         email: user_info.email,
//         role: body.role.clone(),
//         joined_at: chrono::Utc::now().into(),
//     })
// }
//
// // 7. 프로젝트에서 멤버 제거
// #[delete("/projects/{project_id}/members/{user_id}")]
// pub async fn remove_member(
//     db: web::Data<sea_orm::DatabaseConnection>,
//     path: web::Path<(i32, i32)>,
//     auth_user: web::ReqData<i32>,
// ) -> impl Responder {
//     let (project_id, target_user_id) = path.into_inner();
//     let user_id = *auth_user.into_inner();
//
//     // 자신을 제거하려는 경우 (프로젝트 탈퇴)
//     let is_self_removal = user_id == target_user_id;
//
//     if !is_self_removal {
//         // 사용자가 프로젝트의 관리자인지 확인
//         let is_admin = ProjectMemberEntity::find()
//             .filter(
//                 Condition::all()
//                     .add(project_member::Column::ProjectId.eq(project_id))
//                     .add(project_member::Column::UserId.eq(user_id))
//                     .add(project_member::Column::Role.is_in(vec!["owner", "admin"]))
//             )
//             .one(db.get_ref())
//             .await
//             .unwrap()
//             .is_some();
//
//         if !is_admin {
//             return HttpResponse::Forbidden().body("You don't have permission to remove members");
//         }
//
//         // 대상이 소유자인지 확인
//         let target_is_owner = ProjectMemberEntity::find()
//             .filter(
//                 Condition::all()
//                     .add(project_member::Column::ProjectId.eq(project_id))
//                     .add(project_member::Column::UserId.eq(target_user_id))
//                     .add(project_member::Column::Role.eq("owner"))
//             )
//             .one(db.get_ref())
//             .await
//             .unwrap()
//             .is_some();
//
//         if target_is_owner {
//             return HttpResponse::Forbidden().body("Project owner cannot be removed");
//         }
//     }
//
//     // 프로젝트 멤버 제거
//     let result = ProjectMemberEntity::delete_many()
//         .filter(
//             Condition::all()
//                 .add(project_member::Column::ProjectId.eq(project_id))
//                 .add(project_member::Column::UserId.eq(target_user_id))
//         )
//         .exec(db.get_ref())
//         .await
//         .unwrap();
//
//     if result.rows_affected > 0 {
//         HttpResponse::NoContent().finish()
//     } else {
//         HttpResponse::NotFound().body("Member not found")
//     }
// }
