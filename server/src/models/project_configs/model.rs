use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use vulcanum_shared::api_types::AgentBackend;

use crate::models::providers::model::{
    IntegrationColumn, IntegrationProject, IntegrationType, IntegrationWorkspace,
};
use crate::util::serde::deserialize_nullable_string;

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
    pub max_turns: i32,
    pub prompt_template: Option<String>,
    pub repo_url: String,
    pub repo_full_names: Vec<String>,
    pub repo_urls: Vec<String>,
    pub agents_md: Option<String>,
    pub primary_model_provider_key: Option<String>,
    pub primary_model_id: Option<String>,
    pub small_model_provider_key: Option<String>,
    pub small_model_id: Option<String>,
    pub review_enabled: Option<bool>,
    pub review_max_turns: Option<i32>,
    pub review_prompt_template: Option<String>,
    pub max_in_progress_tasks: Option<i32>,
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
    #[serde(default = "default_max_turns")]
    pub max_turns: i32,
    #[serde(default)]
    pub prompt_template: Option<String>,
    #[serde(default)]
    pub repo_full_names: Vec<String>,
    #[serde(default)]
    pub agents_md: Option<String>,
    #[serde(default)]
    pub primary_model_provider_key: Option<String>,
    #[serde(default)]
    pub primary_model_id: Option<String>,
    #[serde(default)]
    pub small_model_provider_key: Option<String>,
    #[serde(default)]
    pub small_model_id: Option<String>,
    #[serde(default)]
    pub review_enabled: Option<bool>,
    #[serde(default)]
    pub review_max_turns: Option<i32>,
    #[serde(default)]
    pub review_prompt_template: Option<String>,
    #[serde(default)]
    pub max_in_progress_tasks: Option<i32>,
    #[serde(default)]
    pub integration_type: IntegrationType,
    pub provider_id: Uuid,
}

#[derive(Debug, Default, Deserialize)]
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
    pub max_turns: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub prompt_template: Option<Option<String>>,
    #[serde(default)]
    pub repo_full_names: Option<Vec<String>>,
    #[serde(default)]
    pub agents_md: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub primary_model_provider_key: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub primary_model_id: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub small_model_provider_key: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub small_model_id: Option<Option<String>>,
    #[serde(default)]
    pub review_enabled: Option<Option<bool>>,
    #[serde(default)]
    pub review_max_turns: Option<Option<i32>>,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub review_prompt_template: Option<Option<String>>,
    #[serde(default)]
    pub max_in_progress_tasks: Option<Option<i32>>,
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
    pub fn job_fields(&self, settings: EffectiveProjectSettings) -> JobConfigFields {
        JobConfigFields {
            team_id: self.team_id,
            external_project_id: self.external_project_id.clone(),
            external_workspace_id: self.external_workspace_id.clone(),
            primary_model_provider_key: settings.primary_model_provider_key,
            primary_model_id: settings.primary_model_id,
            small_model_provider_key: settings.small_model_provider_key,
            small_model_id: settings.small_model_id,
            max_turns: self.max_turns,
            review_max_turns: settings.review_max_turns,
            provider_id: self.provider_id,
            agent_backend: settings.agent_backend,
            repo_urls: self.repo_urls.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EffectiveProjectSettings {
    pub prompt_template: String,
    pub agents_md: String,
    pub primary_model_provider_key: Option<String>,
    pub primary_model_id: Option<String>,
    pub small_model_provider_key: Option<String>,
    pub small_model_id: Option<String>,
    pub review_enabled: bool,
    pub review_max_turns: i32,
    pub review_prompt_template: String,
    pub max_in_progress_tasks: i32,
    pub agent_backend: AgentBackend,
}

impl EffectiveProjectSettings {
    #[must_use]
    pub fn empty_for_team(_team_id: Uuid) -> Self {
        Self {
            prompt_template: String::new(),
            agents_md: String::new(),
            primary_model_provider_key: None,
            primary_model_id: None,
            small_model_provider_key: None,
            small_model_id: None,
            review_enabled: false,
            review_max_turns: 0,
            review_prompt_template: String::new(),
            max_in_progress_tasks: 0,
            agent_backend: AgentBackend::default(),
        }
    }
}

pub struct JobConfigFields {
    pub team_id: Uuid,
    pub external_project_id: String,
    pub external_workspace_id: String,
    pub primary_model_provider_key: Option<String>,
    pub primary_model_id: Option<String>,
    pub small_model_provider_key: Option<String>,
    pub small_model_id: Option<String>,
    pub max_turns: i32,
    pub review_max_turns: i32,
    pub provider_id: Option<Uuid>,
    pub repo_urls: Vec<String>,
    pub agent_backend: AgentBackend,
}

impl JobConfigFields {
    pub fn empty_for_team(team_id: Uuid) -> Self {
        Self {
            team_id,
            external_project_id: String::new(),
            external_workspace_id: String::new(),
            primary_model_provider_key: None,
            primary_model_id: None,
            small_model_provider_key: None,
            small_model_id: None,
            max_turns: 0,
            review_max_turns: 1,
            provider_id: None,
            repo_urls: Vec::new(),
            agent_backend: AgentBackend::default(),
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

fn default_max_turns() -> i32 {
    3
}
