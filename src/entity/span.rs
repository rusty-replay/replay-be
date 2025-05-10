use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::api::trace::calculate_duration;
use crate::entity::base_time::ActiveModelTimeBehavior;
use crate::entity::span;

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

impl ActiveModel {
    /// Span ActiveModel 생성자
    pub fn new(
        transaction_id: i32,
        span_id: Vec<u8>,
        parent_span_id: Option<Vec<u8>>,
        name: impl Into<String>,
        start_timestamp: DateTime<Utc>,
        end_timestamp: DateTime<Utc>,
        http_method: Option<impl Into<String>>,
        http_url: Option<impl Into<String>>,
        http_status_code: Option<i32>,
        http_status_text: Option<impl Into<String>>,
        http_response_content_length: Option<i64>,
        http_host: Option<impl Into<String>>,
        http_scheme: Option<impl Into<String>>,
        http_user_agent: Option<impl Into<String>>,
        attributes: Option<Value>,
    ) -> Self {
        let duration_ms = calculate_duration(&start_timestamp, &end_timestamp);

        span::ActiveModel {
            transaction_id: Set(transaction_id),
            span_id: Set(span_id),
            parent_span_id: Set(parent_span_id),
            name: Set(name.into()),
            start_timestamp: Set(start_timestamp),
            end_timestamp: Set(end_timestamp),
            duration_ms: Set(duration_ms),
            http_method: Set(http_method.map(|s| s.into())),
            http_url: Set(http_url.map(|s| s.into())),
            http_status_code: Set(http_status_code),
            http_status_text: Set(http_status_text.map(|s| s.into())),
            http_response_content_length: Set(http_response_content_length),
            http_host: Set(http_host.map(|s| s.into())),
            http_scheme: Set(http_scheme.map(|s| s.into())),
            http_user_agent: Set(http_user_agent.map(|s| s.into())),
            attributes: Set(attributes),
            ..Default::default()
        }
    }
}