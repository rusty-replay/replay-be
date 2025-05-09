use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "span")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub transaction_id: i32,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub start_timestamp: DateTime<Utc>,
    pub end_timestamp: DateTime<Utc>,
    pub duration_ms: i32,
    // HTTP Attributes
    pub http_method: Option<String>,
    pub http_url: Option<String>,
    pub http_status_code: Option<i32>,
    pub http_status_text: Option<String>,
    pub http_response_content_length: Option<i64>,
    pub http_host: Option<String>,
    pub http_scheme: Option<String>,
    pub http_user_agent: Option<String>,
    // 기타 속성
    pub attributes: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
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

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::Transaction => Entity::belongs_to(super::transaction::Entity)
                .from(Column::TransactionId)
                .to(super::transaction::Column::Id)
                .into(),
        }
    }
}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef { Relation::Transaction.def() }
}
