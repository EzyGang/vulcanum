use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkRunsError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
