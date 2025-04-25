use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use crate::entity::error_log::Model as ErrorLogModel;

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

impl From<ErrorLogModel> for ErrorReportListResponse {
    fn from(model: ErrorLogModel) -> Self {
        Self {
            id: model.id,
            message: model.message,
            stacktrace: model.stacktrace,
            app_version: model.app_version,
            timestamp: model.timestamp,
            group_hash: model.group_hash,
            issue_id: model.issue_id,
            browser: model.browser,
            os: model.os,
        }
    }
}

// impl<T, U> Into<U> for T where U: From<T> {
//     fn into(self) -> U {
//         U::from(self)
//     }
// }

impl From<ErrorLogModel> for ErrorReportResponse {
    fn from(model: ErrorLogModel) -> Self {
        Self {
            id: model.id,
            message: model.message,
            stacktrace: model.stacktrace,
            app_version: model.app_version,
            timestamp: model.timestamp,
            group_hash: model.group_hash,
            replay: model.replay,
            environment: model.environment,
            browser: model.browser,
            os: model.os,
            ip_address: model.ip_address,
            user_agent: model.user_agent,
            project_id: model.project_id,
            issue_id: model.issue_id,
            created_at: model.created_at.to_string(),
            updated_at: model.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BatchErrorReportRequest {
    pub events: Vec<ErrorReportRequest>,
}


#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BatchErrorReportResponse {
    pub processed: usize,
    pub success: usize,
    pub errors: Vec<String>,
}
