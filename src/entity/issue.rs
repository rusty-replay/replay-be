use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "issues")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub group_hash: String,
    pub status: String,  // "open", "in_progress", "resolved", "ignored"
    pub first_seen: DateTimeWithTimeZone,
    pub last_seen: DateTimeWithTimeZone,
    pub count: i32,
    pub project_id: i32,
    pub assigned_to: Option<i32>,  // 담당 사용자 ID
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id"
    )]
    Project,

    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::AssignedTo",
        to = "super::user::Column::Id"
    )]
    User,

    #[sea_orm(has_many = "super::event::Entity")]
    Event,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Event.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}