use thiserror::Error;

#[derive(Debug, Error)]
pub enum TeamsError {
    #[error("team not found")]
    NotFound,
    #[error("team access denied")]
    AccessDenied,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
