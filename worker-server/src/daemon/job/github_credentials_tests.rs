use vulcanum_shared::api::error::ApiError;

use crate::daemon::job::github_credentials::is_retryable_refresh_error;

#[test]
fn client_errors_without_retry_semantics_stop_refresh_loop() {
    assert!(!is_retryable_refresh_error(&api_error(400)));
    assert!(!is_retryable_refresh_error(&api_error(403)));
    assert!(!is_retryable_refresh_error(&api_error(404)));
    assert!(!is_retryable_refresh_error(&api_error(409)));
}

#[test]
fn transient_http_errors_keep_refresh_loop_retrying() {
    assert!(is_retryable_refresh_error(&api_error(408)));
    assert!(is_retryable_refresh_error(&api_error(429)));
    assert!(is_retryable_refresh_error(&api_error(500)));
    assert!(is_retryable_refresh_error(&api_error(503)));
}

#[test]
fn non_http_errors_keep_refresh_loop_retrying() {
    let error = anyhow::anyhow!("network timeout");

    assert!(is_retryable_refresh_error(&error));
}

fn api_error(status: u16) -> anyhow::Error {
    ApiError {
        status,
        body: String::new(),
    }
    .into()
}
