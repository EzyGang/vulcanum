pub mod project_configs;

use crate::services::project_configs::errors::ProjectConfigsError;
use sqlx::{Executor, Postgres};

pub trait Queryer<'c>: Executor<'c, Database = Postgres> {}

impl<'c> Queryer<'c> for &sqlx::PgPool {}

impl<'c> Queryer<'c> for &'c mut sqlx::PgConnection {}

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

#[derive(Clone)]
pub struct ProjectConfigsRepository {}

impl ProjectConfigsRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod project_configs_tests;
