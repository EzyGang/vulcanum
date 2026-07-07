mod conversions;
mod provider_conversions;
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

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
    #[error("at least one repository is required")]
    RepositoriesRequired,
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
            Self::RepositoriesRequired => HttpResponse::BadRequest().json(ErrorBody {
                error: "At least one repository is required".to_owned(),
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

#[derive(serde::Serialize)]
struct ErrorBody {
    error: String,
}
