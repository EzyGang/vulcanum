pub mod queries;

use uuid::Uuid;

use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::providers::model::IntegrationType;

fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .map(|db_err| db_err.constraint() == Some("project_configs_team_provider_external_key"))
        .unwrap_or(false)
}

fn map_sqlx_error(err: sqlx::Error) -> ProjectConfigsError {
    if is_unique_violation(&err) {
        ProjectConfigsError::DuplicateExternalProjectId
    } else {
        ProjectConfigsError::Database(err)
    }
}

pub struct UpdateProjectConfigParams<'a> {
    pub name: Option<&'a str>,
    pub pickup_column: Option<&'a str>,
    pub target_column: Option<&'a str>,
    pub progress_column: Option<&'a str>,
    pub max_turns: Option<i32>,
    pub prompt_template: Option<Option<&'a str>>,
    pub repo_url: Option<&'a str>,
    pub agents_md: Option<Option<&'a str>>,
    pub review_enabled: Option<Option<bool>>,
    pub review_max_turns: Option<Option<i32>>,
    pub review_prompt_template: Option<Option<&'a str>>,
    pub max_in_progress_tasks: Option<Option<i32>>,
    pub external_workspace_id: Option<&'a str>,
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
