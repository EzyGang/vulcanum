use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkersError {
    #[error("registration code not found")]
    CodeNotFound,
    #[error("registration code expired")]
    CodeExpired,
    #[error("invalid refresh token")]
    InvalidRefreshToken,
    #[error("refresh token expired")]
    RefreshTokenExpired,
    #[error("worker not found")]
    WorkerNotFound,
    #[error("registration failed: {0}")]
    RegistrationFailed(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
}
