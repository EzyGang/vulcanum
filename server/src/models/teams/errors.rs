use thiserror::Error;

#[derive(Debug, Error)]
pub enum TeamsError {
    #[error("team not found")]
    NotFound,
    #[error("team access denied")]
    AccessDenied,
    #[error("invalid team operation: {0}")]
    InvalidOperation(String),
    #[error("invalid or expired invite")]
    InviteInvalid,
    #[error("invite store error: {0}")]
    InviteStore(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
}
