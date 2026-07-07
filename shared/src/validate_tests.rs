use std::{
    ffi::OsString,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use crate::api_types::AgentBackend;
use crate::config::{IsolationBackend, WorkerConfig};
use crate::validate::{validate_environment, Severity, ValidationIssue};

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct PathOverride {
    previous_path: Option<OsString>,
    temp_dir: PathBuf,
    _lock: MutexGuard<'static, ()>,
}

impl PathOverride {
    fn empty(name: &str) -> Self {
        let lock = ENV_LOCK
            .lock()
            .expect("PATH override lock should not be poisoned");
        let temp_dir = std::env::temp_dir().join(format!(
            "vulcanum-validate-empty-path-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("empty PATH directory should be created");

        let previous_path = std::env::var_os("PATH");
        std::env::set_var("PATH", &temp_dir);

        Self {
            previous_path,
            temp_dir,
            _lock: lock,
        }
    }

    #[cfg(unix)]
    fn executable(&self, name: &str, contents: &[u8]) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let binary_path = self.temp_dir.join(name);
        std::fs::write(&binary_path, contents).expect("fake executable should be written");
        std::fs::set_permissions(&binary_path, std::fs::Permissions::from_mode(0o755))
            .expect("fake executable should be executable");
        binary_path
    }
}

impl Drop for PathOverride {
    fn drop(&mut self) {
        match self.previous_path.as_ref() {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

#[test]
fn docker_backend_requires_missing_docker_as_critical_issue() {
    let _path = PathOverride::empty("docker");

    let issues = validate_environment("docker", AgentBackend::OpenCode);

    assert_has_critical_issue(&issues, "docker not found in PATH");
}

#[cfg(unix)]
#[test]
fn docker_backend_requires_reachable_daemon_not_just_docker_binary() {
    let path = PathOverride::empty("docker-info-fails");
    path.executable("docker", b"#!/bin/sh\nexit 42\n");

    let issues = validate_environment("docker", AgentBackend::OpenCode);

    assert_has_critical_issue(
        &issues,
        "docker daemon is not reachable by the current user or passwordless sudo",
    );
}

#[test]
fn unknown_isolation_backend_is_a_critical_validation_issue() {
    let issues = validate_environment("firecracker", AgentBackend::OpenCode);

    assert_has_critical_issue(
        &issues,
        "unknown isolation backend \"firecracker\"; expected one of: host, docker, kata",
    );
}

#[test]
fn worker_config_isolation_backend_parses_known_harnesses_and_rejects_unknown() {
    let cases = [
        ("host", IsolationBackend::Host),
        ("docker", IsolationBackend::Docker),
        ("kata", IsolationBackend::Kata),
    ];

    for (harness, expected) in cases {
        let config = WorkerConfig {
            harness: harness.to_owned(),
            ..WorkerConfig::default()
        };

        assert_eq!(config.isolation_backend().unwrap(), expected);
    }

    let config = WorkerConfig {
        harness: "firecracker".to_owned(),
        ..WorkerConfig::default()
    };
    let err = config
        .isolation_backend()
        .expect_err("unknown harness should be rejected");

    assert_eq!(err.value(), "firecracker");
    assert_eq!(
        err.to_string(),
        "unknown isolation backend \"firecracker\"; expected one of: host, docker, kata"
    );
}

#[test]
fn host_backend_requires_missing_selected_agent_binary_as_critical_issue() {
    let _path = PathOverride::empty("host-agent");
    let cases = [
        (AgentBackend::OpenCode, "opencode not found in PATH"),
        (AgentBackend::OmpRpc, "omp not found in PATH"),
    ];

    for (agent_backend, expected_message) in cases {
        let issues = validate_environment("host", agent_backend);

        assert_has_critical_issue(&issues, expected_message);
    }
}

#[cfg(unix)]
#[test]
fn host_backend_treats_non_executable_matching_agent_binary_as_missing() {
    use std::os::unix::fs::PermissionsExt;

    let path = PathOverride::empty("non-executable-opencode");
    let binary_path = path.temp_dir.join("opencode");
    std::fs::write(&binary_path, b"not executable")
        .expect("non-executable agent binary placeholder should be written");
    std::fs::set_permissions(&binary_path, std::fs::Permissions::from_mode(0o644))
        .expect("agent binary placeholder should be made non-executable");

    let issues = validate_environment("host", AgentBackend::OpenCode);

    assert_has_critical_issue(&issues, "opencode not found in PATH");
}

fn assert_has_critical_issue(issues: &[ValidationIssue], expected_message: &str) {
    assert!(
        issues.iter().any(|issue| {
            issue.severity == Severity::Critical && issue.message == expected_message
        }),
        "expected critical issue {expected_message:?}, got {issues:?}"
    );
}
