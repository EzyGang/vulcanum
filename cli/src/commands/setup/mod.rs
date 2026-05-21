use crate::harness::validate::validate_environment;
use crate::harness::validate::Severity;

mod docker;
mod image;
mod kata;
mod opencode;
mod systemd;
mod utils;

pub async fn run() -> anyhow::Result<()> {
    tracing::info!("starting worker environment provisioning...");

    run_step("install_docker", || docker::install_docker().map(|_| ()))?;
    run_step("install_kata", || kata::install_kata().map(|_| ()))?;
    run_step("pull_agent_image", || image::pull_agent_image().map(|_| ()))?;
    run_step("verify_opencode", || {
        opencode::verify_or_install_opencode().map(|_| ())
    })?;
    run_step("configure_systemd", systemd::configure_systemd)?;

    tracing::info!("running final environment validation...");
    let issues = validate_environment();

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

    if critical > 0 {
        tracing::error!(
            "{} critical issues remain — verify installation and re-run `vulcanum worker setup`",
            critical
        );
        anyhow::bail!(
            "provisioning incomplete: {} critical issues remain",
            critical
        );
    }

    tracing::info!(
        "provisioning complete ({} warnings) — worker environment is ready",
        warnings
    );
    Ok(())
}

fn run_step(name: &str, step: impl FnOnce() -> anyhow::Result<()>) -> anyhow::Result<()> {
    tracing::info!("[{name}] running...");
    match step() {
        Ok(()) => {
            tracing::info!("[{name}] done");
            Ok(())
        }
        Err(e) => {
            tracing::error!("[{name}] failed: {e:#}");
            tracing::error!(
                "provisioning failed at step '{name}' — fix the issue and re-run `vulcanum worker setup`"
            );
            Err(e)
        }
    }
}
