use thiserror::Error;

use crate::services::users::errors::UsersError;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid token")]
    InvalidToken,
    #[error(transparent)]
    Users(#[from] UsersError),
}
