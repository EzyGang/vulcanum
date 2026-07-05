use std::{
    ffi::OsString,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use crate::api_types::AgentBackend;
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

fn assert_has_critical_issue(issues: &[ValidationIssue], expected_message: &str) {
    assert!(
        issues.iter().any(|issue| {
            issue.severity == Severity::Critical && issue.message == expected_message
        }),
        "expected critical issue {expected_message:?}, got {issues:?}"
    );
}
