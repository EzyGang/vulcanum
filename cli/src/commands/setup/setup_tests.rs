use crate::commands::setup::host::capacity_from_resources;
use crate::commands::setup::prompts::resolve_backend;
use crate::commands::setup::{Backend, InteractionMode};
use crate::{console, IsolationBackend};

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
