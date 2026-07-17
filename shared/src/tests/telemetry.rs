use std::process::Command;

use tracing_test::traced_test;

#[traced_test]
#[test]
fn test_info_event_is_captured_with_structured_fields() {
    let worker_id = "w1";
    tracing::info!(worker_id = worker_id, duration_ms = 42, "test event");
    assert!(logs_contain("test event"));
    assert!(logs_contain("worker_id"));
    assert!(logs_contain("w1"));
    assert!(logs_contain("duration_ms"));
    assert!(logs_contain("42"));
}

#[traced_test]
#[test]
fn test_warning_event_is_captured() {
    tracing::warn!(error = "something went wrong", "warning event");
    assert!(logs_contain("warning event"));
    assert!(logs_contain("error"));
    assert!(logs_contain("something went wrong"));
}

#[traced_test]
#[test]
fn test_error_event_is_captured() {
    tracing::error!(exit_code = 1, "failure event");
    assert!(logs_contain("failure event"));
    assert!(logs_contain("exit_code"));
    assert!(logs_contain("1"));
}

#[test]
fn try_init_with_config_is_idempotent_after_successful_initialization() {
    const CHILD_ENV: &str = "VULCANUM_TELEMETRY_IDEMPOTENT_CHILD";

    if std::env::var_os(CHILD_ENV).is_some() {
        crate::telemetry::try_init_with_config(false, None)
            .expect("fresh process should initialize telemetry");
        crate::telemetry::try_init_with_config(true, Some("json"))
            .expect("second initialization should be a no-op");
        return;
    }

    let status = Command::new(std::env::current_exe().expect("test executable should resolve"))
        .env(CHILD_ENV, "1")
        .arg("--exact")
        .arg("tests::telemetry::try_init_with_config_is_idempotent_after_successful_initialization")
        .status()
        .expect("child telemetry test should run");

    assert!(status.success(), "child telemetry test should pass");
}
