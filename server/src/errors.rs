use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

use crate::services::auth::errors::AuthError;
use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::users::errors::UsersError;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::workers::errors::WorkersError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("user not found")]
    UserNotFound,
    #[error("invalid token")]
    InvalidToken,
    #[error("authorization header required")]
    AuthHeaderMissing,
    #[error("invalid password")]
    InvalidPassword,
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
    #[error("internal server error")]
    Internal,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
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
            Self::Internal => HttpResponse::InternalServerError().json(ErrorBody {
                error: "Internal server error".to_owned(),
            }),
        }
    }
}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidToken => Self::InvalidToken,
            AuthError::InvalidPassword => Self::InvalidPassword,
            AuthError::Users(u) => u.into(),
        }
    }
}

impl From<UsersError> for AppError {
    fn from(err: UsersError) -> Self {
        match err {
            UsersError::UserNotFound => Self::UserNotFound,
            UsersError::Database(e) => {
                tracing::error!("{e}");
                Self::Internal
            }
        }
    }
}

impl From<ProjectConfigsError> for AppError {
    fn from(err: ProjectConfigsError) -> Self {
        match err {
            ProjectConfigsError::NotFound => Self::ProjectConfigNotFound,
            ProjectConfigsError::DuplicateKaneoProjectId => Self::DuplicateProjectConfig,
            ProjectConfigsError::Database(e) => {
                tracing::error!("{e}");
                Self::Internal
            }
            ProjectConfigsError::Kaneo(e) => {
                tracing::error!("{e}");
                Self::Internal
            }
            ProjectConfigsError::ColumnNotFound(e) => {
                tracing::error!("{e}");
                Self::Internal
            }
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
                tracing::error!("{e}");
                Self::Internal
            }
            WorkRunsError::Dispatch(e) => {
                tracing::error!("{e}");
                Self::Internal
            }
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
                tracing::error!("{e}");
                Self::Internal
            }
            WorkersError::Jwt(e) => {
                tracing::error!("{e}");
                Self::Internal
            }
            WorkersError::Redis(e) => {
                tracing::error!("{e}");
                Self::Internal
            }
        }
    }
}

#[derive(serde::Serialize)]
struct ErrorBody {
    error: String,
}
