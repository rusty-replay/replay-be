use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::project::ProjectCreateRequest;
use crate::entity::base_time::{ActiveModelTimeBehavior, BaseTimeFields};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "transaction")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub trace_id: String,
    pub name: String,
    pub start_timestamp: DateTime<Utc>,
    pub end_timestamp: DateTime<Utc>,
    pub duration_ms: i32,
    pub environment: String,
    pub status: String,
    pub tags: Option<Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id"
    )]
    Project,
    #[sea_orm(has_many = "super::span::Entity")]
    Span,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef { Relation::Project.def() }
}
impl Related<super::span::Entity> for Entity {
    fn to() -> RelationDef { Relation::Span.def() }
}



#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}