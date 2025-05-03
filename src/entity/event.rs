use chrono::{Utc, DateTime};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::event::EventReportRequest;
use crate::entity::base_time::{BaseTimeFields, ActiveModelTimeBehavior};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "event")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub message: String,
    pub stacktrace: String,
    pub app_version: String,
    pub timestamp: DateTime<Utc>,
    pub group_hash: String,
    pub replay: Option<Value>,
    pub environment: String,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub project_id: i32,
    pub issue_id: Option<i32>,
    pub reported_by: Option<i32>,
    pub additional_info: Option<Value>,

    pub priority: Option<Priority>,

    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "priority")]
pub enum Priority {
    #[sea_orm(string_value = "HIGH")]
    HIGH,
    #[sea_orm(string_value = "MED")]
    MED,
    #[sea_orm(string_value = "LOW")]
    LOW,
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

impl BaseTimeFields for Model {
    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    fn updated_at(&self) -> &Option<DateTime<Utc>> {
        &self.updated_at
    }
    fn deleted_at(&self) -> &Option<DateTime<Utc>> {
        &self.deleted_at
    }
    fn deleted_by(&self) -> &Option<i64> {
        &self.deleted_by
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C: ConnectionTrait>(self, db: &C, insert: bool) -> Result<Self, DbErr> {
        self.before_save_common(db, insert).await
    }
}

impl ActiveModelTimeBehavior for ActiveModel {
    fn set_created_at(&mut self, dt: DateTime<Utc>) {
        self.created_at = Set(dt);
    }

    fn set_updated_at(&mut self, dt: DateTime<Utc>) {
        self.updated_at = Set(Some(dt));
    }

    fn set_deleted(&mut self, by: i64, dt: DateTime<Utc>) {
        self.deleted_at = Set(Some(dt));
        self.deleted_by = Set(Some(by));
    }

    fn clear_deleted(&mut self) {
        self.deleted_at = Set(None);
        self.deleted_by = Set(None);
    }
}

impl ActiveModel {
    pub fn from_error_event(
        event: &EventReportRequest,
        project_id: i32,
        issue_id: i32,
        group_hash: String,
    ) -> Self {
        let timestamp = event.timestamp;

        Self {
            message: Set(event.message.clone()),
            stacktrace: Set(event.stacktrace.clone()),
            app_version: Set(event.app_version.clone()),
            timestamp: Set(timestamp),
            group_hash: Set(group_hash),
            replay: Set(event.replay.clone()),
            environment: Set(event.environment.clone().unwrap_or_else(|| "production".to_string())),
            browser: Set(event.browser.clone()),
            os: Set(event.os.clone()),
            ip_address: Set(None),
            user_agent: Set(event.user_agent.clone()),
            project_id: Set(project_id),
            issue_id: Set(Some(issue_id)),
            reported_by: Set(event.user_id),
            additional_info: Set(event.additional_info.clone()),
            ..Default::default()
        }
    }

    pub fn soft_delete(&mut self, deleted_by: i64) {
        let now = Utc::now();
        self.set_deleted(deleted_by, now);
    }

    pub fn restore(&mut self) {
        self.clear_deleted();
    }
}
