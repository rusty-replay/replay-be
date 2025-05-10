use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::entity::base_time::ActiveModelTimeBehavior;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "span")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub transaction_id: i32,

    pub span_id: Vec<u8>,
    pub parent_span_id: Option<Vec<u8>>,

    pub name: String,
    pub start_timestamp: DateTime<Utc>,
    pub end_timestamp: DateTime<Utc>,
    pub duration_ms: i32,

    pub http_method: Option<String>,
    pub http_url: Option<String>,
    pub http_status_code: Option<i32>,
    pub http_status_text: Option<String>,
    pub http_response_content_length: Option<i64>,
    pub http_host: Option<String>,
    pub http_scheme: Option<String>,
    pub http_user_agent: Option<String>,

    pub attributes: Option<Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::TransactionId",
        to = "super::transaction::Column::Id"
    )]
    Transaction,
}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef { Relation::Transaction.def() }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}