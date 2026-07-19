use crate::runtime::errors::HarnessError;

#[test]
fn display_install() {
    let e = HarnessError::Install("missing binary".to_owned());
    assert_eq!(e.to_string(), "install error: missing binary");
}

#[test]
fn display_timeout() {
    let e = HarnessError::Timeout(1_800);
    assert_eq!(e.to_string(), "job timed out after 1800s");
}

#[test]
fn display_crash() {
    let e = HarnessError::Crash("segfault".to_owned());
    assert_eq!(e.to_string(), "agent crashed: segfault");
}

#[test]
fn display_output_parse() {
    let e = HarnessError::OutputParse("no pr url".to_owned());
    assert_eq!(e.to_string(), "output parse error: no pr url");
}

#[test]
fn display_server_launch() {
    let e = HarnessError::ServerLaunch("bind failed".to_owned());
    assert_eq!(e.to_string(), "server launch failed: bind failed");
}

#[test]
fn display_server_unhealthy() {
    let e = HarnessError::ServerUnhealthy("timeout".to_owned());
    assert_eq!(e.to_string(), "server unhealthy: timeout");
}

#[test]
fn display_stall_detected() {
    let e = HarnessError::StallDetected(300);
    assert_eq!(e.to_string(), "stall detected: no event for 300s");
}

#[test]
fn display_cancel_failed() {
    let e = HarnessError::CancelFailed("not running".to_owned());
    assert_eq!(e.to_string(), "cancel failed: not running");
}

#[test]
fn display_http() {
    let e = HarnessError::Http("connection refused".to_owned());
    assert_eq!(e.to_string(), "http error: connection refused");
}
