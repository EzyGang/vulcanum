use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct AppProject {
    pub id: Uuid,
    pub external_project_id: String,
    pub name: String,
    pub external_workspace_id: String,
    pub enabled: bool,
    #[serde(default)]
    pub pickup_column: String,
    #[serde(default)]
    pub progress_column: String,
    #[serde(default)]
    pub review_column: String,
    #[serde(default)]
    pub done_column: String,
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

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct UpdateProjectRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pickup_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_full_names: Option<Vec<String>>,
}
