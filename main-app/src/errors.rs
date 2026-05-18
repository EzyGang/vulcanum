use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

use crate::services::auth::errors::AuthError;
use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::users::errors::UsersError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("user not found")]
    UserNotFound,
    #[error("invalid token")]
    InvalidToken,
    #[error("project config not found")]
    ProjectConfigNotFound,
    #[error("duplicate project config")]
    DuplicateProjectConfig,
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
            Self::ProjectConfigNotFound => HttpResponse::NotFound().json(ErrorBody {
                error: "Project config not found".to_owned(),
            }),
            Self::DuplicateProjectConfig => HttpResponse::Conflict().json(ErrorBody {
                error: "A config for this project already exists".to_owned(),
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
            AuthError::Users(u) => u.into(),
        }
    }
}

impl From<UsersError> for AppError {
    fn from(err: UsersError) -> Self {
        match err {
            UsersError::UserNotFound => Self::UserNotFound,
            UsersError::Database(_) => Self::Internal,
        }
    }
}

impl From<ProjectConfigsError> for AppError {
    fn from(err: ProjectConfigsError) -> Self {
        match err {
            ProjectConfigsError::NotFound => Self::ProjectConfigNotFound,
            ProjectConfigsError::DuplicateKaneoProjectId => Self::DuplicateProjectConfig,
            ProjectConfigsError::Database(_) | ProjectConfigsError::Kaneo(_) => Self::Internal,
            ProjectConfigsError::ColumnNotFound(_) => Self::Internal,
        }
    }
}

#[derive(serde::Serialize)]
struct ErrorBody {
    error: String,
}
