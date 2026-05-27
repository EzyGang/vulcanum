use crate::console;

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
