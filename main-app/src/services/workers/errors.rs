use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkersError {
    #[error("registration code not found")]
    CodeNotFound,
    #[error("registration code expired")]
    CodeExpired,
    #[error("invalid refresh token")]
    InvalidRefreshToken,
    #[error("worker not found")]
    #[allow(dead_code)]
    WorkerNotFound,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
}
