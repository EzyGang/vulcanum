#[cfg(unix)]
use std::{
    ffi::OsString,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

#[cfg(unix)]
use crate::commands::setup::docker_daemon::docker_runtime_registered;
use crate::commands::setup::host::capacity_from_resources;
use crate::commands::setup::prompts::resolve_backend;
use crate::commands::setup::{Backend, InteractionMode};
use crate::{console, IsolationBackend};

#[cfg(unix)]
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[cfg(unix)]
struct FakeDockerPath {
    previous_path: Option<OsString>,
    previous_runtimes: Option<OsString>,
    temp_dir: PathBuf,
    _lock: MutexGuard<'static, ()>,
}

#[cfg(unix)]
impl FakeDockerPath {
    fn new(name: &str) -> Self {
        use std::os::unix::fs::PermissionsExt;

        let lock = ENV_LOCK
            .lock()
            .expect("environment override lock should not be poisoned");
        let temp_dir = std::env::temp_dir().join(format!(
            "vulcanum-fake-docker-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("fake docker directory should be created");

        let docker_path = temp_dir.join("docker");
        std::fs::write(
            &docker_path,
            b"#!/bin/sh\nprintf '%s\\n' \"$VULCANUM_FAKE_DOCKER_RUNTIMES\"\n",
        )
        .expect("fake docker executable should be written");
        std::fs::set_permissions(&docker_path, std::fs::Permissions::from_mode(0o755))
            .expect("fake docker executable should be executable");

        let previous_path = std::env::var_os("PATH");
        let previous_runtimes = std::env::var_os("VULCANUM_FAKE_DOCKER_RUNTIMES");
        std::env::set_var("PATH", &temp_dir);

        Self {
            previous_path,
            previous_runtimes,
            temp_dir,
            _lock: lock,
        }
    }
}

#[cfg(unix)]
impl Drop for FakeDockerPath {
    fn drop(&mut self) {
        match self.previous_path.as_ref() {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }
        match self.previous_runtimes.as_ref() {
            Some(runtimes) => std::env::set_var("VULCANUM_FAKE_DOCKER_RUNTIMES", runtimes),
            None => std::env::remove_var("VULCANUM_FAKE_DOCKER_RUNTIMES"),
        }
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

#[test]
fn test_step_ok() {
    let result = console::step("test", || Ok::<_, anyhow::Error>(42));
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_step_err() {
    let result = console::step("test", || Err::<(), _>(anyhow::anyhow!("test error")));
    assert!(result.is_err());
}

#[test]
fn test_severity_discrimination() {
    use vulcanum_shared::validate::{Severity, ValidationIssue};

    let issues = [
        ValidationIssue {
            severity: Severity::Critical,
            message: String::new(),
        },
        ValidationIssue {
            severity: Severity::Warning,
            message: String::new(),
        },
    ];

    let critical_count = issues
        .iter()
        .filter(|i| i.severity == Severity::Critical)
        .count();
    let warning_count = issues
        .iter()
        .filter(|i| i.severity == Severity::Warning)
        .count();

    assert_eq!(critical_count, 1);
    assert_eq!(warning_count, 1);
}

#[cfg(unix)]
#[test]
fn docker_runtime_registered_matches_exact_runtime_keys() {
    let _docker = FakeDockerPath::new("runtime-json");

    std::env::set_var(
        "VULCANUM_FAKE_DOCKER_RUNTIMES",
        r#"{"kata-runtime-old":{"path":"/usr/bin/kata-runtime"}}"#,
    );
    assert!(!docker_runtime_registered("kata-runtime"));

    std::env::set_var(
        "VULCANUM_FAKE_DOCKER_RUNTIMES",
        r#"{"kata-runtime":{"path":"/usr/bin/kata-runtime"}}"#,
    );
    assert!(docker_runtime_registered("kata-runtime"));
}

#[test]
fn noninteractive_setup_defaults_to_docker_when_isolation_is_omitted() {
    let backend = resolve_backend(InteractionMode::NonInteractive, None)
        .expect("noninteractive setup without isolation should not prompt");

    assert_eq!(backend, Backend::Docker);
}

#[test]
fn explicit_docker_and_none_isolation_select_requested_backend() {
    let cases = [
        (IsolationBackend::Docker, Backend::Docker),
        (IsolationBackend::None, Backend::None),
    ];

    for (isolation, expected) in cases {
        let backend = resolve_backend(InteractionMode::NonInteractive, Some(isolation))
            .expect("explicit Docker or host isolation should resolve without probing KVM");

        assert_eq!(backend, expected);
    }
}

#[test]
fn capacity_caps_at_three_jobs() {
    assert_eq!(capacity_from_resources(32, 128 * 1024 * 1024), 3);
}

#[test]
fn capacity_has_minimum_one_job() {
    assert_eq!(capacity_from_resources(1, 512 * 1024), 1);
}

#[test]
fn capacity_uses_lower_cpu_or_memory_limit() {
    assert_eq!(capacity_from_resources(16, 8 * 1024 * 1024), 2);
}
