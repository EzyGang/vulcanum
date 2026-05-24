use crate::console;
use crate::harness::validate::validate_environment;
use crate::harness::validate::Severity;

mod docker;
mod image;
mod kata;
mod opencode;
mod systemd;
mod utils;

#[cfg(test)]
mod kata_tests;
#[cfg(test)]
mod mod_tests;

pub async fn run() -> anyhow::Result<()> {
    console::info("Provisioning worker environment...");

    console::step("Docker", docker::install_docker)?;
    console::step("Kata Containers", kata::install_kata)?;
    console::step("Agent image", image::pull_agent_image)?;
    console::step("OpenCode", opencode::verify_or_install_opencode)?;
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
    Ok(())
}
