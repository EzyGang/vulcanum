use thiserror::Error;

use crate::models::provider_configs::errors::IntegrationProvidersError;
use crate::models::providers::errors::IntegrationError;

#[derive(Debug, Error)]
pub enum TaskBoardError {
    #[error("provider error: {0}")]
    Provider(#[from] IntegrationProvidersError),
    #[error("integration error: {0}")]
    Integration(#[from] IntegrationError),
    #[error("task title is required")]
    EmptyTitle,
    #[error("task status is required")]
    EmptyStatus,
}
