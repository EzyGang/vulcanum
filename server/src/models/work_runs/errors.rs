use crate::models::dispatcher::errors::DispatchError;
use crate::models::github_app::errors::GithubAppError;
use crate::models::model_providers::errors::ModelProvidersError;
use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::provider_configs::errors::IntegrationProvidersError;
use crate::models::providers::errors::IntegrationError;
use crate::models::teams::errors::TeamsError;
use crate::models::workers::errors::WorkersError;

#[derive(Debug, thiserror::Error)]
pub enum WorkRunsError {
    #[error("work run not found")]
    NotFound,
    #[error("work run already claimed by another worker")]
    AlreadyClaimed,
    #[error("invalid status transition")]
    InvalidStatusTransition,
    #[error("work run not owned by this worker")]
    NotOwned,
    #[error("cannot delete a running work run")]
    DeleteRunning,
    #[error("invalid pagination: {0}")]
    InvalidPagination(String),
    #[error("failed to update task lifecycle label")]
    LifecycleLabelUpdate,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("dispatch error: {0}")]
    Dispatch(#[from] DispatchError),
    #[error("github app error: {0}")]
    GithubApp(#[from] GithubAppError),
    #[error("model provider error: {0}")]
    ModelProvider(#[from] ModelProvidersError),
    #[error("integration provider configuration error: {0}")]
    ProviderConfig(#[from] IntegrationProvidersError),
    #[error("integration provider error: {0}")]
    Provider(#[from] IntegrationError),
    #[error("project config error: {0}")]
    ProjectConfig(#[from] ProjectConfigsError),
    #[error("team error: {0}")]
    Team(#[from] TeamsError),
    #[error("worker error: {0}")]
    Worker(#[from] WorkersError),
}
