use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectConfigsError {
    #[error("project config not found")]
    NotFound,
    #[error("a config for this kaneo project already exists")]
    DuplicateKaneoProjectId,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("kaneo API error: {0}")]
    Kaneo(#[from] crate::services::kaneo::errors::KaneoError),
    #[error("column not found: {0}")]
    ColumnNotFound(String),
}
