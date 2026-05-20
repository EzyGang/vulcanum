use crate::harness::errors::HarnessError;
use crate::harness::validate::{validate_environment, Severity};

pub async fn run() -> anyhow::Result<()> {
    tracing::info!("validating worker environment...");

    let issues = validate_environment();

    if issues.is_empty() {
        tracing::info!("all checks passed — worker environment is ready");
        return Ok(());
    }

    let mut critical = 0;
    let mut warnings = 0;

    for issue in &issues {
        match issue.severity {
            Severity::Critical => {
                critical += 1;
                tracing::error!("{}", issue.message);
            }
            Severity::Warning => {
                warnings += 1;
                tracing::warn!("{}", issue.message);
            }
        }
    }

    tracing::info!(
        "validation complete: {} critical, {} warnings",
        critical,
        warnings
    );

    if critical > 0 {
        tracing::error!(
            "critical issues found — install required components before running the daemon"
        );
        return Err(HarnessError::Install("critical environment issues found".to_owned()).into());
    }

    Ok(())
}
