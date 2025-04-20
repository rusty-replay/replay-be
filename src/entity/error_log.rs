use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "error_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub message: String,
    pub stacktrace: String,
    pub app_version: String,
    pub timestamp: String,
    pub group_hash: String,
    pub replay: Json,
    pub environment: String,  // "development", "staging", "production"
    pub browser: Option<String>,
    pub os: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub project_id: i32,
    pub issue_id: Option<i32>,  // 이슈와 연결
    pub reported_by: Option<i32>,
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
        belongs_to = "super::issue::Entity",
        from = "Column::IssueId",
        to = "super::issue::Column::Id"
    )]
    Issue,

    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::ReportedBy",
        to = "super::user::Column::Id"
    )]
    ReportedByUser,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::issue::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Issue.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ReportedByUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}