pub mod project_configs;

use uuid::Uuid;

use crate::services::integrations::model::IntegrationType;
use crate::services::project_configs::errors::ProjectConfigsError;

fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .map(|db_err| db_err.constraint() == Some("project_configs_kaneo_project_id_key"))
        .unwrap_or(false)
}

fn map_sqlx_error(err: sqlx::Error) -> ProjectConfigsError {
    if is_unique_violation(&err) {
        ProjectConfigsError::DuplicateKaneoProjectId
    } else {
        ProjectConfigsError::Database(err)
    }
}

pub struct UpdateProjectConfigParams<'a> {
    pub pickup_column: Option<&'a str>,
    pub target_column: Option<&'a str>,
    pub progress_column: Option<&'a str>,
    pub blocked_column: Option<&'a str>,
    pub max_turns: Option<i32>,
    pub prompt_template: Option<&'a str>,
    pub repo_url: Option<&'a str>,
    pub agents_md: Option<&'a str>,
    pub opencode_config: Option<&'a str>,
    pub kaneo_workspace_id: Option<&'a str>,
    pub enabled: Option<bool>,
    pub integration_type: Option<IntegrationType>,
    pub provider_id: Option<Uuid>,
}

#[derive(Clone)]
pub struct ProjectConfigsRepository {}

impl Default for ProjectConfigsRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectConfigsRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod project_configs_tests;
