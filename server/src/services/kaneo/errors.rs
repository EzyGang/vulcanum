use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum KaneoError {
    #[error("kaneo API error: {0}")]
    Api(String),
    #[cfg(test)]
    #[error("column not found in project: {0}")]
    ColumnNotFound(String),
}

pub(crate) fn api_err(e: impl std::fmt::Display) -> KaneoError {
    KaneoError::Api(e.to_string())
}
