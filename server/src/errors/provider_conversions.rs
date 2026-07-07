use crate::models::github_app::errors::GithubAppError;
use crate::models::model_providers::errors::ModelProvidersError;
use crate::models::task_board::errors::TaskBoardError;
use crate::models::work_run_events::errors::WorkRunEventsError;

use super::AppError;

impl From<TaskBoardError> for AppError {
    fn from(err: TaskBoardError) -> Self {
        match err {
            TaskBoardError::Provider(e) => e.into(),
            TaskBoardError::Integration(e) => match provider_request_error(&e) {
                Some(err) => err,
                None => {
                    tracing::error!(error = %e, operation = "task_board", "integration error");
                    Self::Internal
                }
            },
            TaskBoardError::ProjectConfig(e) => e.into(),
            TaskBoardError::WorkRuns(e) => e.into(),
            TaskBoardError::EmptyTitle => Self::BadRequest("Task title is required".to_owned()),
            TaskBoardError::EmptyStatus => Self::BadRequest("Task status is required".to_owned()),
            TaskBoardError::EmptyLabel => Self::BadRequest("Task label is required".to_owned()),
        }
    }
}

fn provider_request_error(
    err: &crate::models::providers::errors::IntegrationError,
) -> Option<AppError> {
    let status = err.provider_http_status_code()?;
    if !(400..500).contains(&status) {
        return None;
    }

    let message = err
        .provider_public_message()
        .filter(|message| !message.trim().is_empty())
        .unwrap_or("Provider rejected request");
    Some(AppError::BadRequest(message.to_owned()))
}

impl From<ModelProvidersError> for AppError {
    fn from(err: ModelProvidersError) -> Self {
        match err {
            ModelProvidersError::NotFound => Self::ModelProviderNotFound,
            ModelProvidersError::DuplicateProvider => Self::DuplicateModelProvider,
            ModelProvidersError::UnknownProvider(provider) => {
                Self::BadRequest(format!("Unknown model provider: {provider}"))
            }
            ModelProvidersError::UnknownModel {
                provider_key,
                model_id,
            } => Self::BadRequest(format!("Unknown model: {provider_key}/{model_id}")),
            ModelProvidersError::Catalog(e) => {
                tracing::error!(error = %e, operation = "model_providers", "catalog error");
                Self::Internal
            }
            ModelProvidersError::InvalidAuthConfig(message) => Self::BadRequest(message),
            ModelProvidersError::DeviceFlowExpired => {
                Self::BadRequest("Device flow expired".to_owned())
            }
            ModelProvidersError::DeviceFlowPending { next_poll_at } => {
                Self::BadRequest(format!("Device flow pending until {next_poll_at}"))
            }
            ModelProvidersError::DeviceFlowFailed(e) => {
                tracing::error!(error = %e, operation = "model_providers", "device flow error");
                Self::Internal
            }
            ModelProvidersError::SecretEncryption(e) => {
                tracing::error!(error = %e, operation = "model_providers", "secret encryption error");
                Self::Internal
            }
            ModelProvidersError::SecretDecryption => {
                tracing::error!(operation = "model_providers", "secret decryption error");
                Self::Internal
            }
            ModelProvidersError::OAuthRefreshFailed(e) => {
                tracing::error!(error = %e, operation = "model_providers", "oauth refresh error");
                Self::Internal
            }
            ModelProvidersError::Database(e) => {
                tracing::error!(error = %e, operation = "model_providers", "database error");
                Self::Internal
            }
            ModelProvidersError::Redis(e) => {
                tracing::error!(error = %e, operation = "model_providers", "redis error");
                Self::Internal
            }
        }
    }
}

impl From<WorkRunEventsError> for AppError {
    fn from(err: WorkRunEventsError) -> Self {
        match err {
            WorkRunEventsError::NotFound => Self::WorkRunNotFound,
            WorkRunEventsError::Database(e) => {
                tracing::error!(error = %e, operation = "work_run_events", "database error");
                Self::Internal
            }
            WorkRunEventsError::CancelStore(msg) => {
                tracing::error!(operation = "work_run_events", "cancel store error: {msg}");
                Self::Internal
            }
            WorkRunEventsError::Internal(msg) => {
                tracing::error!(operation = "work_run_events", "internal error: {msg}");
                Self::Internal
            }
        }
    }
}

impl From<GithubAppError> for AppError {
    fn from(err: GithubAppError) -> Self {
        match err {
            GithubAppError::NoInstallation => {
                Self::BadRequest("No GitHub installation configured".to_owned())
            }
            GithubAppError::InstallationAlreadyLinked => Self::Forbidden,
            GithubAppError::NotConfigured => {
                Self::BadRequest("GitHub App not configured".to_owned())
            }
            GithubAppError::InvalidRepoUrl(url) => {
                Self::BadRequest(format!("Invalid repo URL: {url}"))
            }
            GithubAppError::InvalidRepoIdentifier(identifier) => Self::BadRequest(format!(
                "Invalid GitHub repository identifier: {identifier}"
            )),
            GithubAppError::Api(msg) => {
                tracing::error!(error = %msg, operation = "github_app", "github api error");
                Self::Internal
            }
            GithubAppError::Base64Decode(msg) => {
                Self::BadRequest(format!("Invalid base64 private key: {msg}"))
            }
            GithubAppError::Database(e) => {
                tracing::error!(error = %e, operation = "github_app", "database error");
                Self::Internal
            }
            GithubAppError::Redis(e) => {
                tracing::error!(error = %e, operation = "github_app", "redis error");
                Self::Internal
            }
        }
    }
}
