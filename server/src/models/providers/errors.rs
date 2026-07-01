use thiserror::Error;

use crate::services::providers::kaneo::errors::KaneoError;

#[derive(Debug, Error)]
pub enum IntegrationError {
    #[error("kaneo error: {0}")]
    Kaneo(#[from] KaneoError),
    #[error("integration error: {0}")]
    Other(String),
}

impl IntegrationError {
    pub(crate) fn provider_http_status_code(&self) -> Option<u16> {
        match self {
            Self::Kaneo(error) => error.http_status_code(),
            Self::Other(_) => None,
        }
    }

    pub(crate) fn provider_public_message(&self) -> Option<&str> {
        match self {
            Self::Kaneo(error) => error.public_message(),
            Self::Other(_) => None,
        }
    }
}
