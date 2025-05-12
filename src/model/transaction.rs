use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use crate::entity::transaction;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
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
    pub created_at: DateTime<Utc>,
}

impl From<transaction::Model> for TransactionResponse {
    fn from(model: transaction::Model) -> Self {
        TransactionResponse {
            id: model.id,
            project_id: model.project_id,
            trace_id: model.trace_id,
            name: model.name,
            start_timestamp: model.start_timestamp,
            end_timestamp: model.end_timestamp,
            duration_ms: model.duration_ms,
            environment: model.environment,
            status: model.status,
            tags: model.tags,
            created_at: model.created_at,
        }
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct TraceRequest {
    #[schema(example = "base64 encoded bytes")]
    pub data: String,
}
