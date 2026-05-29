use thiserror::Error;

#[derive(Debug, Error)]
pub enum IntegrationProvidersError {
    #[error("provider not found")]
    NotFound,
    #[error("a provider with this name already exists")]
    DuplicateName,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
