use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use crate::entity::event::{EventStatus, Model as EventModel, Priority};

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventReportRequest {
    pub message: String,
    pub stacktrace: String,
    pub app_version: String,
    pub timestamp: DateTime<Utc>,
    pub replay: Option<Value>,
    pub environment: Option<String>, // "development", "staging", "production"
    pub browser: Option<String>,
    pub os: Option<String>,
    pub user_agent: Option<String>,
    pub api_key: String, // 프로젝트 API 키
    pub user_id: Option<i32>, // 에러가 발생한 사용자 ID
    pub additional_info: Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventReportResponse {
    pub id: i32,
    pub message: String,
    pub stacktrace: String,
    pub app_version: String,
    pub timestamp: DateTime<Utc>,
    pub group_hash: String,
    pub replay: Option<Value>,
    pub environment: String,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub project_id: i32,
    pub issue_id: Option<i32>,
    pub additional_info: Option<Value>,
    pub created_at: String,
    pub updated_at: Option<String>,

    pub priority: Option<Priority>,
    pub assigned_to: Option<i32>,
    pub status: EventStatus,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventReportListResponse {
    pub id: i32,
    pub message: String,
    pub stacktrace: String,
    pub app_version: String,
    pub timestamp: DateTime<Utc>,
    pub group_hash: String,
    pub issue_id: Option<i32>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub has_replay: bool,
    pub priority: Option<Priority>,
    pub assigned_to: Option<i32>,
    pub status: EventStatus,
}

impl From<EventModel> for EventReportListResponse {
    fn from(model: EventModel) -> Self {
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
            has_replay: model.replay.is_some(),
            priority: model.priority,
            assigned_to: model.assigned_to,
            status: model.status,
        }
    }
}

// impl<T, U> Into<U> for T where U: From<T> {
//     fn into(self) -> U {
//         U::from(self)
//     }
// }

impl From<EventModel> for EventReportResponse {
    fn from(model: EventModel) -> Self {
        Self {
            id: model.id,
            message: model.message,
            stacktrace: model.stacktrace,
            app_version: model.app_version,
            timestamp: model.timestamp.into(),
            group_hash: model.group_hash,
            replay: model.replay,
            environment: model.environment,
            browser: model.browser,
            os: model.os,
            ip_address: model.ip_address,
            user_agent: model.user_agent,
            project_id: model.project_id,
            issue_id: model.issue_id,
            additional_info: model.additional_info,
            created_at: model.created_at.to_string(),
            updated_at: model.updated_at.map(|dt| dt.to_string()),
            priority: model.priority,
            assigned_to: model.assigned_to,
            status: model.status,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BatchEventReportRequest {
    pub events: Vec<EventReportRequest>,
}


#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BatchEventReportResponse {
    pub processed: usize,
    pub success: usize,
    pub events: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventQuery {
    pub search: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub content: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total_elements: u64,
    pub filtered_elements: u64,
    pub total_pages: u32,
    pub has_next: bool,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventPriority {
    pub event_ids: Vec<i32>,
    pub priority: Priority,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventAssignee {
    pub event_ids: Vec<i32>,
    pub assigned_to: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventStatusDto {
    pub event_ids: Vec<i32>,
    pub status: EventStatus,
}