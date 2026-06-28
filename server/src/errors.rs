use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

use crate::models::auth::errors::AuthError;
use crate::models::github_app::errors::GithubAppError;
use crate::models::model_providers::errors::ModelProvidersError;
use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::provider_configs::errors::IntegrationProvidersError;
use crate::models::task_board::errors::TaskBoardError;
use crate::models::teams::errors::TeamsError;
use crate::models::users::errors::UsersError;
use crate::models::work_run_events::errors::WorkRunEventsError;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::workers::errors::WorkersError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("user not found")]
    UserNotFound,
    #[error("invalid token")]
    InvalidToken,
    #[error("authorization header required")]
    AuthHeaderMissing,
    #[error("invalid password")]
    InvalidPassword,
    #[error("instance login is disabled")]
    InstanceLoginDisabled,
    #[error("registration code not found")]
    CodeNotFound,
    #[error("registration code expired")]
    CodeExpired,
    #[error("invalid refresh token")]
    InvalidRefreshToken,
    #[error("worker not found")]
    WorkerNotFound,
    #[error("project config not found")]
    ProjectConfigNotFound,
    #[error("duplicate project config")]
    DuplicateProjectConfig,
    #[error("work run not found")]
    WorkRunNotFound,
    #[error("work run already claimed")]
    AlreadyClaimed,
    #[error("work run not owned by this worker")]
    NotOwned,
    #[error("invalid status transition")]
    InvalidStatusTransition,
    #[error("cannot delete a running work run")]
    CannotDeleteRunning,
    #[error("provider not found")]
    ProviderNotFound,
    #[error("duplicate provider name")]
    DuplicateProviderName,
    #[error("model provider not found")]
    ModelProviderNotFound,
    #[error("duplicate model provider")]
    DuplicateModelProvider,
    #[error("column not found")]
    ColumnNotFound,
    #[error("no provider configured")]
    NoProvider,
    #[error("internal server error")]
    Internal,
    #[error("forbidden")]
    Forbidden,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::BadRequest(msg) => {
                HttpResponse::BadRequest().json(ErrorBody { error: msg.clone() })
            }
            Self::UserNotFound => HttpResponse::NotFound().json(ErrorBody {
                error: "User not found".to_owned(),
            }),
            Self::InvalidToken => HttpResponse::Unauthorized().json(ErrorBody {
                error: "Invalid token".to_owned(),
            }),
            Self::AuthHeaderMissing => HttpResponse::BadRequest().json(ErrorBody {
                error: "Authorization header required".to_owned(),
            }),
            Self::InvalidPassword => HttpResponse::Unauthorized().json(ErrorBody {
                error: "Invalid password".to_owned(),
            }),
            Self::InstanceLoginDisabled => HttpResponse::Forbidden().json(ErrorBody {
                error: "Instance login is disabled".to_owned(),
            }),
            Self::CodeNotFound => HttpResponse::BadRequest().json(ErrorBody {
                error: "Registration code not found".to_owned(),
            }),
            Self::CodeExpired => HttpResponse::BadRequest().json(ErrorBody {
                error: "Registration code expired".to_owned(),
            }),
            Self::InvalidRefreshToken => HttpResponse::Unauthorized().json(ErrorBody {
                error: "Invalid refresh token".to_owned(),
            }),
            Self::WorkerNotFound => HttpResponse::NotFound().json(ErrorBody {
                error: "Worker not found".to_owned(),
            }),
            Self::ProjectConfigNotFound => HttpResponse::NotFound().json(ErrorBody {
                error: "Project config not found".to_owned(),
            }),
            Self::DuplicateProjectConfig => HttpResponse::Conflict().json(ErrorBody {
                error: "A config for this project already exists".to_owned(),
            }),
            Self::WorkRunNotFound => HttpResponse::NotFound().json(ErrorBody {
                error: "Work run not found".to_owned(),
            }),
            Self::AlreadyClaimed => HttpResponse::Conflict().json(ErrorBody {
                error: "Work run already claimed".to_owned(),
            }),
            Self::NotOwned => HttpResponse::Forbidden().json(ErrorBody {
                error: "Work run not owned by this worker".to_owned(),
            }),
            Self::InvalidStatusTransition => HttpResponse::Conflict().json(ErrorBody {
                error: "Invalid status transition".to_owned(),
            }),
            Self::CannotDeleteRunning => HttpResponse::Conflict().json(ErrorBody {
                error: "Cannot delete a running work run".to_owned(),
            }),
            Self::ProviderNotFound => HttpResponse::NotFound().json(ErrorBody {
                error: "Provider not found".to_owned(),
            }),
            Self::DuplicateProviderName => HttpResponse::Conflict().json(ErrorBody {
                error: "A provider with this name already exists".to_owned(),
            }),
            Self::ModelProviderNotFound => HttpResponse::NotFound().json(ErrorBody {
                error: "Model provider not found".to_owned(),
            }),
            Self::DuplicateModelProvider => HttpResponse::Conflict().json(ErrorBody {
                error: "A model provider for this team is already connected".to_owned(),
            }),
            Self::ColumnNotFound => HttpResponse::BadRequest().json(ErrorBody {
                error: "Column not found in project".to_owned(),
            }),
            Self::NoProvider => HttpResponse::BadRequest().json(ErrorBody {
                error: "No provider configured for this project".to_owned(),
            }),
            Self::Internal => HttpResponse::InternalServerError().json(ErrorBody {
                error: "Internal server error".to_owned(),
            }),
            Self::Forbidden => HttpResponse::Forbidden().json(ErrorBody {
                error: "Forbidden".to_owned(),
            }),
        }
    }
}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidToken => Self::InvalidToken,
            AuthError::InvalidRefreshToken => Self::InvalidRefreshToken,
            AuthError::InvalidPassword => Self::InvalidPassword,
            AuthError::InstanceLoginDisabled => Self::InstanceLoginDisabled,
            AuthError::Database(e) => {
                tracing::error!(error = %e, operation = "auth", "database error");
                Self::Internal
            }
            AuthError::Users(u) => u.into(),
            AuthError::Teams(t) => t.into(),
        }
    }
}

impl From<UsersError> for AppError {
    fn from(err: UsersError) -> Self {
        match err {
            UsersError::UserNotFound => Self::UserNotFound,
            UsersError::Database(e) => {
                tracing::error!(error = %e, operation = "users", "database error");
                Self::Internal
            }
        }
    }
}

impl From<TeamsError> for AppError {
    fn from(err: TeamsError) -> Self {
        match err {
            TeamsError::NotFound => Self::Forbidden,
            TeamsError::AccessDenied => Self::Forbidden,
            TeamsError::InvalidOperation(message) => Self::BadRequest(message),
            TeamsError::InviteInvalid => Self::BadRequest("Invalid or expired invite".to_owned()),
            TeamsError::InviteStore(e) => {
                tracing::error!(error = %e, operation = "teams", "invite store error");
                Self::Internal
            }
            TeamsError::Database(e) => {
                tracing::error!(error = %e, operation = "teams", "database error");
                Self::Internal
            }
            TeamsError::Redis(e) => {
                tracing::error!(error = %e, operation = "teams", "redis error");
                Self::Internal
            }
        }
    }
}

impl From<ProjectConfigsError> for AppError {
    fn from(err: ProjectConfigsError) -> Self {
        match err {
            ProjectConfigsError::NotFound => Self::ProjectConfigNotFound,
            ProjectConfigsError::DuplicateExternalProjectId => Self::DuplicateProjectConfig,
            ProjectConfigsError::Database(e) => {
                tracing::error!(error = %e, operation = "project_configs", "database error");
                Self::Internal
            }
            ProjectConfigsError::Integration(e) => {
                tracing::error!(error = %e, operation = "project_configs", "integration error");
                Self::Internal
            }
            ProjectConfigsError::ColumnNotFound(_) => Self::ColumnNotFound,
            ProjectConfigsError::NoProvider => Self::NoProvider,
            ProjectConfigsError::ModelProvider(e) => e.into(),
            ProjectConfigsError::Team(e) => e.into(),
        }
    }
}

impl From<WorkRunsError> for AppError {
    fn from(err: WorkRunsError) -> Self {
        match err {
            WorkRunsError::NotFound => Self::WorkRunNotFound,
            WorkRunsError::AlreadyClaimed => Self::AlreadyClaimed,
            WorkRunsError::NotOwned => Self::NotOwned,
            WorkRunsError::InvalidStatusTransition => Self::InvalidStatusTransition,
            WorkRunsError::Database(e) => {
                tracing::error!(error = %e, operation = "work_runs", "database error");
                Self::Internal
            }
            WorkRunsError::Dispatch(e) => {
                tracing::error!(error = %e, operation = "work_runs", "dispatch error");
                Self::Internal
            }
            WorkRunsError::DeleteRunning => Self::CannotDeleteRunning,
            WorkRunsError::GithubApp(e) => e.into(),
            WorkRunsError::ModelProvider(e) => e.into(),
            WorkRunsError::ProjectConfig(e) => e.into(),
            WorkRunsError::Team(e) => e.into(),
        }
    }
}

impl From<WorkersError> for AppError {
    fn from(err: WorkersError) -> Self {
        match err {
            WorkersError::CodeNotFound => Self::CodeNotFound,
            WorkersError::CodeExpired => Self::CodeExpired,
            WorkersError::InvalidRefreshToken => Self::InvalidRefreshToken,
            WorkersError::RefreshTokenExpired => Self::InvalidRefreshToken,
            WorkersError::WorkerNotFound => Self::WorkerNotFound,
            WorkersError::Database(e) => {
                tracing::error!(error = %e, operation = "workers", "database error");
                Self::Internal
            }
            WorkersError::Jwt(e) => {
                tracing::error!(error = %e, operation = "workers", "jwt error");
                Self::Internal
            }
            WorkersError::Redis(e) => {
                tracing::error!(error = %e, operation = "workers", "redis error");
                Self::Internal
            }
        }
    }
}

impl From<IntegrationProvidersError> for AppError {
    fn from(err: IntegrationProvidersError) -> Self {
        match err {
            IntegrationProvidersError::NotFound => Self::ProviderNotFound,
            IntegrationProvidersError::DuplicateName => Self::DuplicateProviderName,
            IntegrationProvidersError::Database(e) => {
                tracing::error!(error = %e, operation = "providers", "database error");
                Self::Internal
            }
        }
    }
}

impl From<TaskBoardError> for AppError {
    fn from(err: TaskBoardError) -> Self {
        match err {
            TaskBoardError::Provider(e) => e.into(),
            TaskBoardError::Integration(e) => {
                tracing::error!(error = %e, operation = "task_board", "integration error");
                Self::Internal
            }
            TaskBoardError::ProjectConfig(e) => e.into(),
            TaskBoardError::EmptyTitle => Self::BadRequest("Task title is required".to_owned()),
            TaskBoardError::EmptyStatus => Self::BadRequest("Task status is required".to_owned()),
        }
    }
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
                Self::BadRequest("No GitHub installation configured".to_string())
            }
            GithubAppError::InstallationAlreadyLinked => Self::Forbidden,
            GithubAppError::NotConfigured => {
                Self::BadRequest("GitHub App not configured".to_string())
            }
            GithubAppError::InvalidRepoUrl(url) => {
                Self::BadRequest(format!("Invalid repo URL: {url}"))
            }
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

#[derive(serde::Serialize)]
struct ErrorBody {
    error: String,
}
