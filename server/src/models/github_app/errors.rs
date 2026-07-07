use thiserror::Error;

#[derive(Debug, Error)]
pub enum GithubAppError {
    #[error("github app is not configured")]
    NotConfigured,
    #[error("no github installation found")]
    NoInstallation,
    #[error("github installation is already linked to another team")]
    InstallationAlreadyLinked,
    #[error("invalid repo_url: {0}")]
    InvalidRepoUrl(String),
    #[error("invalid github repository identifier: {0}")]
    InvalidRepoIdentifier(String),
    #[error("github api error: {0}")]
    Api(String),
    #[error("invalid base64 private key: {0}")]
    Base64Decode(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("redis error: {0}")]
    Redis(String),
}
