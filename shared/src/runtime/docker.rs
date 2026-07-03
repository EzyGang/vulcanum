use std::fmt;
use std::future::Future;
use std::thread;
use std::time::Duration;

use thiserror::Error;

pub const PULL_ATTEMPTS: u8 = 3;
pub const PULL_RETRY_DELAY: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub struct DockerPullError {
    image: String,
    message: Option<String>,
}

impl DockerPullError {
    fn new(image: &str, message: String) -> Self {
        Self {
            image: image.to_owned(),
            message: Some(message),
        }
    }

    fn without_message(image: &str) -> Self {
        Self {
            image: image.to_owned(),
            message: None,
        }
    }
}

impl fmt::Display for DockerPullError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(message) => write!(f, "docker pull '{}' failed: {message}", self.image),
            None => write!(f, "docker pull '{}' failed", self.image),
        }
    }
}

#[must_use = "the Docker pull result must be handled"]
pub async fn retry_docker_pull<F, Fut>(image: &str, pull: F) -> Result<(), DockerPullError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    retry_docker_pull_with(image, PULL_ATTEMPTS, PULL_RETRY_DELAY, pull).await
}

#[must_use = "the Docker pull result must be handled"]
pub(crate) async fn retry_docker_pull_with<F, Fut>(
    image: &str,
    attempts: u8,
    retry_delay: Duration,
    mut pull: F,
) -> Result<(), DockerPullError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    for attempt in 1..=attempts {
        match pull().await {
            Ok(()) => return Ok(()),
            Err(message) if attempt < attempts => {
                log_retry(attempt, &message);
                tokio::time::sleep(retry_delay).await;
            }
            Err(message) => return Err(DockerPullError::new(image, message)),
        }
    }

    Err(DockerPullError::without_message(image))
}

#[must_use = "the Docker pull result must be handled"]
pub fn retry_docker_pull_blocking<F>(image: &str, pull: F) -> Result<(), DockerPullError>
where
    F: FnMut() -> Result<(), String>,
{
    retry_docker_pull_blocking_with(image, PULL_ATTEMPTS, PULL_RETRY_DELAY, pull)
}

#[must_use = "the Docker pull result must be handled"]
pub(crate) fn retry_docker_pull_blocking_with<F>(
    image: &str,
    attempts: u8,
    retry_delay: Duration,
    mut pull: F,
) -> Result<(), DockerPullError>
where
    F: FnMut() -> Result<(), String>,
{
    for attempt in 1..=attempts {
        match pull() {
            Ok(()) => return Ok(()),
            Err(message) if attempt < attempts => {
                log_retry(attempt, &message);
                thread::sleep(retry_delay);
            }
            Err(message) => return Err(DockerPullError::new(image, message)),
        }
    }

    Err(DockerPullError::without_message(image))
}

fn log_retry(attempt: u8, message: &str) {
    tracing::debug!(
        attempt,
        error = %message,
        "docker pull failed; retrying"
    );
}
