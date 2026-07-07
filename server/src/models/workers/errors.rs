use thiserror::Error;
use uuid::Uuid;

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
    #[error(
        "worker active_jobs invariant violated for {worker_id}: active_jobs was {active_jobs}"
    )]
    ActiveJobsInvariant { worker_id: Uuid, active_jobs: i32 },
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
}
