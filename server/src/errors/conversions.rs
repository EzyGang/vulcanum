use crate::models::auth::errors::AuthError;
use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::provider_configs::errors::IntegrationProvidersError;
use crate::models::teams::errors::TeamsError;
use crate::models::users::errors::UsersError;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::workers::errors::WorkersError;

use super::AppError;

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidToken => Self::InvalidToken,
            AuthError::InvalidRefreshToken => Self::InvalidRefreshToken,
            AuthError::InvalidPassword => Self::InvalidPassword,
            AuthError::InstanceLoginDisabled => Self::InstanceLoginDisabled,
            AuthError::GithubOAuth(e) => {
                tracing::warn!(error = %e, operation = "github_oauth", "oauth flow failed");
                Self::InvalidToken
            }
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
            ProjectConfigsError::RepositoriesRequired => Self::RepositoriesRequired,
            ProjectConfigsError::InvalidAgentBackend(value) => {
                tracing::error!(
                    agent_backend = %value,
                    operation = "project_configs",
                    "invalid stored agent backend"
                );
                Self::Internal
            }
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
            WorkRunsError::InvalidPagination(message) => Self::BadRequest(message),
            WorkRunsError::LifecycleLabelUpdate => Self::Internal,
            WorkRunsError::ReviewTicketCreationPending => Self::Internal,
            WorkRunsError::GithubApp(e) => e.into(),
            WorkRunsError::ModelProvider(e) => e.into(),
            WorkRunsError::ProviderConfig(e) => {
                tracing::error!(error = %e, operation = "work_runs", "provider config error");
                Self::Internal
            }
            WorkRunsError::Provider(e) => {
                tracing::error!(error = %e, operation = "work_runs", "provider error");
                Self::Internal
            }
            WorkRunsError::ProjectConfig(e) => e.into(),
            WorkRunsError::Team(e) => e.into(),
            WorkRunsError::Worker(e) => e.into(),
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
            WorkersError::ActiveJobsInvariant {
                worker_id,
                active_jobs,
            } => {
                tracing::error!(
                    %worker_id,
                    active_jobs,
                    operation = "workers",
                    "active_jobs invariant violation"
                );
                Self::Internal
            }
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
            WorkersError::RegistrationFailed(e) => Self::BadRequest(e),
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
