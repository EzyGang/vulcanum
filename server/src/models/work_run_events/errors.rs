use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkRunEventsError {
    #[error("work run events not found")]
    NotFound,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("cancel store error: {0}")]
    CancelStore(String),
    #[error("internal error: {0}")]
    Internal(String),
}
