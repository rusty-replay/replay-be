use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "project_members")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,

    #[sea_orm(primary_key, auto_increment = false)]
    pub project_id: i32,

    pub role: Role,

    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, Copy, ToSchema)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "role")]
pub enum Role {
    #[sea_orm(string_value = "owner")]
    Owner,

    #[sea_orm(string_value = "admin")]
    Admin,

    #[sea_orm(string_value = "member")]
    Member,

    #[sea_orm(string_value = "viewer")]
    Viewer,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,

    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id"
    )]
    Project,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}