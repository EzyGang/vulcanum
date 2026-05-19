use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum WorkRunsError {
    #[error("work run not found")]
    NotFound,
    #[error("work run already claimed by another worker")]
    AlreadyClaimed,
    #[error("invalid status transition")]
    InvalidStatusTransition,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
