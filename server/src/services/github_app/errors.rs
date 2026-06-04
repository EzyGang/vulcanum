use thiserror::Error;

#[derive(Debug, Error)]
pub enum GithubAppError {
    #[error("github app is not configured")]
    NotConfigured,
    #[error("no github installation found")]
    NoInstallation,
    #[error("invalid repo_url: {0}")]
    InvalidRepoUrl(String),
    #[error("github api error: {0}")]
    Api(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("redis error: {0}")]
    Redis(String),
}
