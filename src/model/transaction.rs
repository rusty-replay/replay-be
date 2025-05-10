use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::entity::transaction;

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub id: i32,
    pub project_id: i32,
    pub trace_id: String,
    pub name: String,
    pub start_timestamp: String,
    pub end_timestamp: String,
    pub duration_ms: i32,
    pub environment: String,
    pub status: String,
    pub tags: Option<Value>,
}

impl From<transaction::Model> for TransactionResponse {
    fn from(model: transaction::Model) -> Self {
        TransactionResponse {
            id: model.id,
            project_id: model.project_id,
            trace_id: model.trace_id,
            name: model.name,
            start_timestamp: model.start_timestamp.to_rfc3339(),
            end_timestamp: model.end_timestamp.to_rfc3339(),
            duration_ms: model.duration_ms,
            environment: model.environment,
            status: model.status,
            tags: model.tags,
        }
    }
}
