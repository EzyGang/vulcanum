use thiserror::Error;

use crate::services::users::errors::UsersError;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid token")]
    InvalidToken,
    #[error("invalid refresh token")]
    InvalidRefreshToken,
    #[error("invalid password")]
    InvalidPassword,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Users(#[from] UsersError),
}
