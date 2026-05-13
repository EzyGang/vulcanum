use thiserror::Error;

#[derive(Debug, Error)]
pub enum UsersError {
    #[error("user not found")]
    UserNotFound,
    #[error("database error")]
    Database(#[from] sqlx::Error),
}
