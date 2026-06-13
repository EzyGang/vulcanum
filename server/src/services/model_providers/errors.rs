use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelProvidersError {
    #[error("model provider not found")]
    NotFound,
    #[error("model provider already connected")]
    DuplicateProvider,
    #[error("provider not found in catalog: {0}")]
    UnknownProvider(String),
    #[error("model not found in catalog: {provider_key}/{model_id}")]
    UnknownModel {
        provider_key: String,
        model_id: String,
    },
    #[error("catalog error: {0}")]
    Catalog(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
