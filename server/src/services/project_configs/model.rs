use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::services::integrations::model::{IntegrationColumn, IntegrationType};

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ProjectConfig {
    pub id: Uuid,
    pub kaneo_project_id: String,
    pub kaneo_workspace_id: String,
    pub integration_type: IntegrationType,
    pub enabled: bool,
    pub pickup_column: String,
    pub target_column: String,
    pub progress_column: String,
    pub blocked_column: String,
    pub max_turns: i32,
    pub prompt_template: String,
    pub repo_url: String,
    pub agents_md: String,
    pub opencode_config: String,
    pub created_at: DateTime<Utc>,
    pub provider_id: Option<Uuid>,
    pub github_token: Option<String>,
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
    #[serde(default = "default_blocked_column")]
    pub blocked_column: String,
    #[serde(default = "default_max_turns")]
    pub max_turns: i32,
    pub prompt_template: String,
    #[serde(default)]
    pub repo_url: String,
    #[serde(default)]
    pub agents_md: String,
    #[serde(default)]
    pub opencode_config: String,
    #[serde(default)]
    pub integration_type: IntegrationType,
    pub provider_id: Uuid,
    #[serde(default)]
    pub github_token: Option<String>,
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
    pub blocked_column: Option<String>,
    #[serde(default)]
    pub max_turns: Option<i32>,
    pub prompt_template: Option<String>,
    #[serde(default)]
    pub repo_url: Option<String>,
    #[serde(default)]
    pub agents_md: Option<String>,
    #[serde(default)]
    pub opencode_config: Option<String>,
    #[serde(default)]
    pub kaneo_workspace_id: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub integration_type: Option<IntegrationType>,
    #[serde(default)]
    pub provider_id: Option<Uuid>,
    #[serde(default)]
    pub github_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LookupProjectResult {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
}

impl From<&IntegrationColumn> for ColumnInfo {
    fn from(col: &IntegrationColumn) -> Self {
        Self {
            id: col.id.clone(),
            name: col.name.clone(),
            slug: col.slug.clone(),
        }
    }
}

impl ProjectConfig {
    pub fn job_fields(&self) -> JobConfigFields {
        JobConfigFields {
            kaneo_project_id: self.kaneo_project_id.clone(),
            kaneo_workspace_id: self.kaneo_workspace_id.clone(),
            opencode_config: self.opencode_config.clone(),
            max_turns: self.max_turns,
            provider_id: self.provider_id,
            github_token: self.github_token.clone(),
        }
    }
}

#[derive(Default)]
pub struct JobConfigFields {
    pub kaneo_project_id: String,
    pub kaneo_workspace_id: String,
    pub opencode_config: String,
    pub max_turns: i32,
    pub provider_id: Option<Uuid>,
    pub github_token: Option<String>,
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

fn default_blocked_column() -> String {
    "Blocked".to_owned()
}

fn default_max_turns() -> i32 {
    3
}
