use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use sea_query::Condition;
use crate::entity::project_member;
use crate::model::global_error::{AppError, ErrorCode};
use crate::entity::project_member::{Entity as ProjectMemberEntity, ActiveModel as ProjectMemberActiveModel};
use crate::entity::project_member::Role as ProjectMemberRole;

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
                .add(project_member::Column::Role.eq(ProjectMemberRole::Owner))
        )
        .one(db)
        .await?;

    if is_owner.is_none() {
        return Err(AppError::forbidden(ErrorCode::NotEnoughPermission));
    }

    Ok(())
}