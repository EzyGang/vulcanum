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

impl KaneoError {
    pub(crate) fn http_status_code(&self) -> Option<u16> {
        match self {
            Self::Api(message) => parse_http_status_code(message),
            #[cfg(test)]
            Self::ColumnNotFound(_) => None,
        }
    }

    pub(crate) fn public_message(&self) -> Option<&str> {
        match self {
            Self::Api(message) => strip_http_status_prefix(message),
            #[cfg(test)]
            Self::ColumnNotFound(message) => Some(message),
        }
    }
}

pub(crate) fn api_err(e: impl std::fmt::Display) -> KaneoError {
    KaneoError::Api(e.to_string())
}

fn parse_http_status_code(message: &str) -> Option<u16> {
    let status = message.get(..3)?.parse::<u16>().ok()?;
    match status {
        400..=599 => Some(status),
        _ => None,
    }
}

fn strip_http_status_prefix(message: &str) -> Option<&str> {
    let status = parse_http_status_code(message)?;
    let rest = message.strip_prefix(&status.to_string())?.trim_start();
    let (_, message) = rest.split_once(": ")?;
    Some(message)
}
