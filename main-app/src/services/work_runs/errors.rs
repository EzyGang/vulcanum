use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum WorkRunsError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
