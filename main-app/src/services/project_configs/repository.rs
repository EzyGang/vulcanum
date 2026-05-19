pub mod project_configs;

use crate::queryer::Queryer;
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
    pub prompt_template: Option<&'a str>,
    pub repo_url: Option<&'a str>,
    pub enabled: Option<bool>,
}

#[derive(Clone)]
pub struct ProjectConfigsRepository {}

impl ProjectConfigsRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod project_configs_tests;
