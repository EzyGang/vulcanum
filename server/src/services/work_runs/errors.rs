use crate::services::dispatcher::errors::DispatchError;
use crate::services::github_app::errors::GithubAppError;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::teams::errors::TeamsError;

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
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("dispatch error: {0}")]
    Dispatch(#[from] DispatchError),
    #[error("github app error: {0}")]
    GithubApp(#[from] GithubAppError),
    #[error("model provider error: {0}")]
    ModelProvider(#[from] ModelProvidersError),
    #[error("project config error: {0}")]
    ProjectConfig(#[from] ProjectConfigsError),
    #[error("team error: {0}")]
    Team(#[from] TeamsError),
}
