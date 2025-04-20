use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectCreateRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectUpdateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInviteRequest {
    pub user_id: i32,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: i32,
    pub name: String,
    pub api_key: String,
    pub description: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMemberResponse {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub role: String,
    pub joined_at: DateTimeWithTimeZone,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDetailResponse {
    pub project: ProjectResponse,
    pub members: Vec<ProjectMemberResponse>,
}