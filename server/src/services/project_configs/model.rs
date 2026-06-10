use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::services::providers::model::{
    IntegrationColumn, IntegrationProject, IntegrationType, IntegrationWorkspace,
};

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ProjectConfig {
    pub id: Uuid,
    pub team_id: Uuid,
    pub external_project_id: String,
    #[serde(default)]
    pub name: String,
    pub external_workspace_id: String,
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
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectConfigRequest {
    pub external_project_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub external_workspace_id: String,
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
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectConfigRequest {
    #[serde(default)]
    pub name: Option<String>,
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
    pub external_workspace_id: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub integration_type: Option<IntegrationType>,
    #[serde(default)]
    pub provider_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct LookupProjectResult {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
}

impl From<IntegrationWorkspace> for WorkspaceInfo {
    fn from(w: IntegrationWorkspace) -> Self {
        Self {
            id: w.id,
            name: w.name,
        }
    }
}

impl From<IntegrationProject> for ProjectInfo {
    fn from(p: IntegrationProject) -> Self {
        Self {
            id: p.id,
            name: p.name,
            slug: p.slug,
        }
    }
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
            team_id: self.team_id,
            external_project_id: self.external_project_id.clone(),
            external_workspace_id: self.external_workspace_id.clone(),
            opencode_config: self.opencode_config.clone(),
            max_turns: self.max_turns,
            provider_id: self.provider_id,
            repo_url: self.repo_url.clone(),
        }
    }
}

pub struct JobConfigFields {
    pub team_id: Uuid,
    pub external_project_id: String,
    pub external_workspace_id: String,
    pub opencode_config: String,
    pub max_turns: i32,
    pub provider_id: Option<Uuid>,
    pub repo_url: String,
}

impl Default for JobConfigFields {
    fn default() -> Self {
        Self {
            team_id: Uuid::nil(),
            external_project_id: String::new(),
            external_workspace_id: String::new(),
            opencode_config: String::new(),
            max_turns: 0,
            provider_id: None,
            repo_url: String::new(),
        }
    }
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
