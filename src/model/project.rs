use serde::{Deserialize, Serialize};
use crate::entity::project::Model as ProjectModel;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProjectCreateRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProjectUpdateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInviteRequest {
    pub user_id: i32,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    pub id: i32,
    pub name: String,
    pub api_key: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<ProjectModel> for ProjectResponse {
    fn from(model: ProjectModel) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            api_key: model.api_key,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMemberResponse {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetailResponse {
    pub project: ProjectResponse,
    pub members: Vec<ProjectMemberResponse>,
}