use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct AppProject {
    pub id: Uuid,
    pub external_project_id: String,
    pub name: String,
    pub external_workspace_id: String,
    pub enabled: bool,
    pub repo_full_names: Vec<String>,
    pub provider_id: Option<Uuid>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct ProviderWorkspace {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct ProviderProject {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CreateProjectRequest {
    pub external_project_id: String,
    pub external_workspace_id: String,
    pub name: String,
    pub provider_id: Uuid,
    pub enabled: bool,
    pub repo_full_names: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UpdateProjectRequest {
    pub repo_full_names: Vec<String>,
}
