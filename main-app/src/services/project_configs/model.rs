use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ProjectConfig {
    pub id: Uuid,
    pub kaneo_project_id: String,
    pub kaneo_workspace_id: String,
    pub enabled: bool,
    pub pickup_column: String,
    pub target_column: String,
    pub progress_column: String,
    pub prompt_template: String,
    pub repo_url: String,
    pub agents_md: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectConfigRequest {
    pub kaneo_project_id: String,
    #[serde(default)]
    pub kaneo_workspace_id: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_pickup_column")]
    pub pickup_column: String,
    #[serde(default = "default_progress_column")]
    pub progress_column: String,
    #[serde(default = "default_target_column")]
    pub target_column: String,
    pub prompt_template: String,
    #[serde(default)]
    pub repo_url: String,
    #[serde(default)]
    pub agents_md: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectConfigRequest {
    #[serde(default)]
    pub pickup_column: Option<String>,
    #[serde(default)]
    pub progress_column: Option<String>,
    #[serde(default)]
    pub target_column: Option<String>,
    #[serde(default)]
    pub prompt_template: Option<String>,
    #[serde(default)]
    pub repo_url: Option<String>,
    #[serde(default)]
    pub agents_md: Option<String>,
    #[serde(default)]
    pub kaneo_workspace_id: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

fn default_enabled() -> bool {
    true
}

fn default_pickup_column() -> String {
    "to-do".to_owned()
}

fn default_progress_column() -> String {
    "in-progress".to_owned()
}

fn default_target_column() -> String {
    "in-review".to_owned()
}
