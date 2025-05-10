use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use crate::entity::span;
use crate::model::transaction::TransactionResponse;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SpanResponse {
    pub id: i32,
    pub transaction_id: i32,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub start_timestamp: String,
    pub end_timestamp: String,
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

impl From<span::Model> for SpanResponse {
    fn from(model: span::Model) -> Self {
        SpanResponse {
            id: model.id,
            transaction_id: model.transaction_id,
            span_id: hex::encode(&model.span_id),
            parent_span_id: model.parent_span_id.map(|p| hex::encode(p)),
            name: model.name,
            start_timestamp: model.start_timestamp.to_rfc3339(),
            end_timestamp: model.end_timestamp.to_rfc3339(),
            duration_ms: model.duration_ms,
            http_method: model.http_method,
            http_url: model.http_url,
            http_status_code: model.http_status_code,
            http_status_text: model.http_status_text,
            http_response_content_length: model.http_response_content_length,
            http_host: model.http_host,
            http_scheme: model.http_scheme,
            http_user_agent: model.http_user_agent,
            attributes: model.attributes,
        }
    }
}


fn default_page() -> i32 { 1 }
fn default_size() -> i32 { 10 }

#[derive(Deserialize, ToSchema)]
pub struct TransactionListQuery {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_size")]
    pub size: i32,
}

#[derive(Serialize, ToSchema)]
pub struct TransactionWithSpansResponse {
    pub transaction: TransactionResponse,
    pub spans: Vec<SpanResponse>,
}