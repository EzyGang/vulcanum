use thiserror::Error;

use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::providers::errors::IntegrationError;
use crate::services::teams::errors::TeamsError;

#[derive(Debug, Error)]
pub enum ProjectConfigsError {
    #[error("project config not found")]
    NotFound,
    #[error("a config for this external project already exists")]
    DuplicateExternalProjectId,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("integration error: {0}")]
    Integration(#[from] IntegrationError),
    #[error("column not found: {0}")]
    ColumnNotFound(String),
    #[error("no provider configured for this project")]
    NoProvider,
    #[error("model provider error: {0}")]
    ModelProvider(#[from] ModelProvidersError),
    #[error("team error: {0}")]
    Team(#[from] TeamsError),
}
