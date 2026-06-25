use thiserror::Error;

use crate::models::workers::errors::WorkersError;

#[derive(Debug, Error)]
pub enum DispatchError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("cancel store error: {0}")]
    Cancel(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error("worker error: {0}")]
    Worker(#[from] WorkersError),
}
