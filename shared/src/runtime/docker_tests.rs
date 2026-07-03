use std::time::Duration;

use super::docker::{retry_docker_pull_blocking_with, retry_docker_pull_with};

#[tokio::test]
async fn async_retry_succeeds_after_transient_failure() {
    let mut attempts = 0;

    retry_docker_pull_with("test-image:v1", 3, Duration::ZERO, || {
        attempts += 1;
        let should_succeed = attempts == 2;
        async move {
            if should_succeed {
                Ok(())
            } else {
                Err("temporary registry error".to_owned())
            }
        }
    })
    .await
    .expect("second pull attempt should succeed");

    assert_eq!(attempts, 2);
}

#[tokio::test]
async fn async_retry_returns_last_failure_after_retries() {
    let mut attempts = 0;

    let error = retry_docker_pull_with("test-image:v1", 3, Duration::ZERO, || {
        attempts += 1;
        async { Err("registry unavailable".to_owned()) }
    })
    .await
    .expect_err("all pull attempts should fail");

    assert_eq!(attempts, 3);
    assert_eq!(
        error.to_string(),
        "docker pull 'test-image:v1' failed: registry unavailable"
    );
}

#[test]
fn blocking_retry_succeeds_after_transient_failure() {
    let mut attempts = 0;

    retry_docker_pull_blocking_with("test-image:v1", 3, Duration::ZERO, || {
        attempts += 1;
        if attempts == 2 {
            Ok(())
        } else {
            Err("temporary registry error".to_owned())
        }
    })
    .expect("second pull attempt should succeed");

    assert_eq!(attempts, 2);
}

#[test]
fn blocking_retry_returns_last_failure_after_retries() {
    let mut attempts = 0;

    let error = retry_docker_pull_blocking_with("test-image:v1", 3, Duration::ZERO, || {
        attempts += 1;
        Err("registry unavailable".to_owned())
    })
    .expect_err("all pull attempts should fail");

    assert_eq!(attempts, 3);
    assert_eq!(
        error.to_string(),
        "docker pull 'test-image:v1' failed: registry unavailable"
    );
}
