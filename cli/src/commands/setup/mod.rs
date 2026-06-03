use vulcanum_shared::validate::{validate_environment_for_backend, Severity};
use vulcanum_shared::worker_state;

use crate::console;

use prompts::resolve_backend;
use registration::{connect_worker, verify_connection};

mod docker;
pub(crate) mod gvisor;
pub(crate) mod image;
mod kata;
mod prompts;
mod registration;
pub(crate) mod systemd;
pub(crate) mod utils;

#[cfg(test)]
mod gvisor_tests;
#[cfg(test)]
mod kata_tests;
#[cfg(test)]
mod mod_tests;

pub async fn run(
    code: Option<String>,
    instance: Option<String>,
    force: bool,
    isolation: Option<crate::IsolationBackend>,
) -> anyhow::Result<()> {
    console::info("Checking prerequisites...");
    utils::has_sudo_access()?;

    console::info("Provisioning worker environment...");

    console::step("Docker", docker::install_docker)?;

    let backend = resolve_backend(interaction_mode(&code, &instance), isolation)?;

    match backend {
        Backend::Kata => {
            console::step("Kata Containers", kata::install_kata)?;
            console::step("Docker Kata runtime", kata::configure_docker_for_kata)?;
        }
        Backend::Gvisor => {
            console::step("gVisor", gvisor::install_gvisor)?;
            console::step("Docker gVisor runtime", gvisor::configure_docker_for_gvisor)?;
        }
        Backend::Docker | Backend::None => {
            console::info("Skipping sandbox runtime installation.");
        }
    }

    console::step("Agent image", image::pull_agent_image)?;
    console::step("Systemd service", || {
        systemd::configure_systemd(backend.harness_name())
    })?;

    eprintln!();
    console::info("Running final environment validation...");
    let issues = validate_environment_for_backend(backend.harness_name());

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

    let harness_name = backend.harness_name();

    if already_connected && !force {
        match verify_connection().await {
            Ok(()) => {
                console::info("Already connected to an instance — connection verified.");
            }
            Err(e) => {
                tracing::warn!("connection verification failed: {e:#}");
                console::warn(&format!(
                    "Connection to instance lost ({e:#}). Re-authenticating..."
                ));
                connect_worker(code, instance).await?;
            }
        }
    } else {
        if force {
            console::info("Forcing re-registration...");
        } else {
            console::info("Registering worker with instance...");
        }
        connect_worker(code, instance).await?;
    }

    eprintln!();
    console::info("Enabling and starting worker service...");
    systemd::enable_and_restart_service()?;

    eprintln!();
    eprintln!(
        "Worker setup complete — daemon is running via systemd with '{harness_name}' harness."
    );
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InteractionMode {
    Interactive,
    NonInteractive,
}

fn interaction_mode(code: &Option<String>, instance: &Option<String>) -> InteractionMode {
    match (code, instance) {
        (Some(_), Some(_)) => InteractionMode::NonInteractive,
        _ => InteractionMode::Interactive,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Backend {
    Kata,
    Gvisor,
    Docker,
    None,
}

impl Backend {
    fn harness_name(&self) -> &'static str {
        match self {
            Self::Kata => "kata",
            Self::Gvisor => "gvisor",
            Self::Docker => "docker",
            Self::None => "host",
        }
    }
}
