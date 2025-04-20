use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct ErrorReportRequest {
    pub message: String,
    pub stacktrace: String,
    pub app_version: String,
    pub timestamp: String,
    pub replay: Value,
}

#[derive(Debug, Serialize)]
pub struct ErrorReportResponse {
    pub id: i32,
    pub message: String,
    pub stacktrace: String,
    pub app_version: String,
    pub timestamp: String,
    pub group_hash: String,
    pub replay: Value,
    pub reported_by: Option<i32>,
}
