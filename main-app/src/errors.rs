use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    InvalidToken,
    UserNotFound,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(e) => write!(f, "Database error: {e}"),
            Self::InvalidToken => write!(f, "Invalid token"),
            Self::UserNotFound => write!(f, "User not found"),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::Database(_) => HttpResponse::InternalServerError().json(ErrorBody {
                error: "Internal server error".to_owned(),
            }),
            Self::InvalidToken => HttpResponse::Unauthorized().json(ErrorBody {
                error: "Invalid token".to_owned(),
            }),
            Self::UserNotFound => HttpResponse::NotFound().json(ErrorBody {
                error: "User not found".to_owned(),
            }),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        Self::Database(e)
    }
}

#[derive(serde::Serialize)]
struct ErrorBody {
    error: String,
}
