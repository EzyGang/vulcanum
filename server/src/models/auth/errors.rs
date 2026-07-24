use thiserror::Error;

use crate::models::github_app::errors::GithubAppError;
use crate::models::teams::errors::TeamsError;
use crate::models::users::errors::UsersError;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid token")]
    InvalidToken,
    #[error("invalid refresh token")]
    InvalidRefreshToken,
    #[error("invalid password")]
    InvalidPassword,
    #[error("instance login is disabled")]
    InstanceLoginDisabled,
    #[error("github oauth failed: {0}")]
    GithubOAuth(String),
    #[error(transparent)]
    GithubApp(#[from] GithubAppError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Users(#[from] UsersError),
    #[error(transparent)]
    Teams(#[from] TeamsError),
}
