use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorReportRequest {
    pub message: String,
    pub stacktrace: String,
    pub app_version: String,
    pub timestamp: String,
    pub replay: Value,
    pub environment: Option<String>, // "development", "staging", "production"
    pub browser: Option<String>,
    pub os: Option<String>,
    pub user_agent: Option<String>,
    pub api_key: String, // 프로젝트 API 키
    pub user_id: Option<i32>, // 에러가 발생한 사용자 ID
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorReportResponse {
    pub id: i32,
    pub message: String,
    pub stacktrace: String,
    pub app_version: String,
    pub timestamp: String,
    pub group_hash: String,
    pub replay: Value,
    pub environment: String,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub project_id: i32,
    pub issue_id: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorReportListResponse {
    pub id: i32,
    pub message: String,
    pub stacktrace: String,
    pub app_version: String,
    pub timestamp: String,
    pub group_hash: String,
    pub issue_id: Option<i32>,
    pub browser: Option<String>,
    pub os: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BatchErrorReportRequest {
    pub events: Vec<ErrorReportRequest>,
}

// #[derive(Debug, Deserialize, ToSchema)]
// #[serde(rename_all = "camelCase")]
// pub struct BatchedEvent {
//     pub id: String,
//     pub timestamp: String,
//     pub message: String,
//     pub stacktrace: String,
//     pub replay: Value,
//     pub environment: String,
//     pub browser: Option<String>,
//     pub os: Option<String>,
//     pub user_agent: Option<String>,
//     pub user_id: Option<i32>,
//     pub additional_info: Option<Value>,
// }

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BatchErrorReportResponse {
    pub processed: usize,
    pub success: usize,
    pub errors: Vec<String>,
}