use thiserror::Error;

use crate::services::kaneo::errors::KaneoError;

#[derive(Debug, Error)]
pub enum IntegrationError {
    #[error("kaneo error: {0}")]
    Kaneo(#[from] KaneoError),
    #[error("integration error: {0}")]
    Other(String),
}
