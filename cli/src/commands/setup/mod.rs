use vulcanum_shared::validate::{validate_environment, Severity};
use vulcanum_shared::worker_state;

use crate::commands::connect;
use crate::console;

mod docker;
pub(crate) mod image;
mod kata;
pub(crate) mod systemd;
mod utils;

#[cfg(test)]
mod kata_tests;
#[cfg(test)]
mod mod_tests;

pub async fn run(
    code: Option<String>,
    instance: Option<String>,
    force: bool,
) -> anyhow::Result<()> {
    console::info("Provisioning worker environment...");

    console::step("Docker", docker::install_docker)?;
    console::step("Kata Containers", kata::install_kata)?;
    console::step("Docker Kata runtime", kata::configure_docker_for_kata)?;
    console::step("Agent image", image::pull_agent_image)?;
    console::step("Systemd service", systemd::configure_systemd)?;

    eprintln!();
    console::info("Running final environment validation...");
    let issues = validate_environment();

    let mut critical = 0;
    let mut warnings = 0;

    for issue in &issues {
        match issue.severity {
            Severity::Critical => {
                critical += 1;
                tracing::error!("{}", issue.message);
                eprintln!("  [CRITICAL] {}", issue.message);
            }
            Severity::Warning => {
                warnings += 1;
                tracing::warn!("{}", issue.message);
                console::warn(&issue.message);
            }
        }
    }

    if critical > 0 {
        anyhow::bail!(
            "provisioning incomplete: {} critical issues remain",
            critical
        );
    }

    eprintln!();
    if warnings > 0 {
        eprintln!("Provisioning complete ({warnings} warnings) — worker environment is ready.");
    } else {
        eprintln!("Provisioning complete — worker environment is ready.");
    }

    let already_connected = worker_state::load_state().ok().flatten().is_some();

    if already_connected && !force {
        console::info("Already connected to an instance — skipping registration.");
    } else if force {
        console::info("Forcing re-registration...");
        connect::run(code, instance).await?;
    } else {
        console::info("Registering worker with instance...");
        connect::run(code, instance).await?;
    }

    eprintln!();
    console::info("Enabling and starting worker service...");
    systemd::configure_systemd()?;
    systemd::enable_and_start_service()?;

    eprintln!();
    eprintln!("Worker setup complete — daemon is running via systemd.");
    Ok(())
}
