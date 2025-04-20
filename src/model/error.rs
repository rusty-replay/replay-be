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
    pub issue_id: Option<i32>, // 연결된 이슈 ID
}