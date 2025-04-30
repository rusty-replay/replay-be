use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use crate::entity::project::Model as ProjectModel;

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
#[serde(rename_all = "camelCase")]
pub struct ProjectInviteRequest {
    pub user_id: i32,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    pub id: i32,
    pub name: String,
    pub api_key: String,
    pub description: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
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