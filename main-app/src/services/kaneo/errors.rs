use thiserror::Error;

#[derive(Debug, Error)]
pub enum KaneoError {
    #[error("kaneo API error: {0}")]
    Api(String),
    #[error("column not found in project: {0}")]
    ColumnNotFound(String),
}

pub(crate) fn api_err(e: impl std::fmt::Display) -> KaneoError {
    KaneoError::Api(e.to_string())
}
