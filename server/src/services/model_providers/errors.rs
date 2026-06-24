use chrono::{DateTime, Utc};
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
    #[error("invalid auth config: {0}")]
    InvalidAuthConfig(String),
    #[error("device flow expired")]
    DeviceFlowExpired,
    #[error("device flow pending until {next_poll_at}")]
    DeviceFlowPending { next_poll_at: DateTime<Utc> },
    #[error("device flow failed: {0}")]
    DeviceFlowFailed(String),
    #[error("secret encryption failed: {0}")]
    SecretEncryption(String),
    #[error("secret decryption failed")]
    SecretDecryption,
    #[error("oauth refresh failed: {0}")]
    OAuthRefreshFailed(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
}
