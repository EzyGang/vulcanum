use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelProvidersError {
    #[error("model provider not found")]
    NotFound,
    #[error("model provider already connected")]
    DuplicateProvider,
    #[error("invalid model provider auth type: {0}")]
    InvalidAuthType(String),
    #[error("invalid model provider selection: {0}")]
    InvalidSelection(String),
    #[error("provider not found in catalog: {0}")]
    UnknownProvider(String),
    #[error("model not found in catalog: {provider_key}/{model_id}")]
    UnknownModel {
        provider_key: String,
        model_id: String,
    },
    #[error("catalog error: {0}")]
    Catalog(String),
    #[error("oauth error: {0}")]
    OAuth(String),
    #[error("credential encryption error")]
    Crypto,
    #[error("serialization error")]
    Serialization,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
