use vulcanum_shared::api_types::AgentBackend;
use vulcanum_shared::validate::Severity;
use vulcanum_shared::worker_state;

use crate::console;
use crate::AgentBackendArg;

use connect::{connect_worker, verify_connection};
use prompts::resolve_backend;

mod backends;
mod connect;
#[cfg(target_os = "linux")]
pub(crate) mod docker_daemon;
pub(crate) mod host;
#[cfg(target_os = "macos")]
mod launchd;
mod prompts;
pub(crate) mod service;
#[cfg(target_os = "linux")]
mod systemd;

#[cfg(test)]
mod setup_tests;

pub async fn run(
    code: Option<String>,
    instance: Option<String>,
    force: bool,
    isolation: Option<crate::IsolationBackend>,
    agent_backend: Option<AgentBackendArg>,
) -> anyhow::Result<()> {
    let backend = resolve_backend(interaction_mode(&code, &instance), isolation)?;

    console::info("Checking prerequisites...");
    host::has_sudo_access()?;

    console::info("Provisioning worker environment...");

    let mut config = vulcanum_shared::config::load_config_if_exists()?.unwrap_or_default();
    config.harness = backend.harness_name().to_owned();
    config.agent_backend = selected_agent_backend(agent_backend);

    match backend {
        Backend::Kata => {
            console::step("Docker", backends::docker::install_docker)?;
            console::step("Kata Containers", backends::kata::install_kata)?;
            console::step(
                "Docker Kata runtime",
                backends::kata::configure_docker_for_kata,
            )?;
            console::step("Agent image", backends::agent_image::pull_agent_image)?;
        }
        Backend::Docker => {
            console::step("Docker", backends::docker::install_docker)?;
            console::info("Skipping sandbox runtime installation.");
            console::step("Agent image", backends::agent_image::pull_agent_image)?;
        }
        Backend::None => {
            console::info("Skipping Docker, sandbox runtime, and agent image installation.");
        }
    }
    console::step(
        service::worker_service_label(),
        service::configure_worker_service,
    )?;

    eprintln!();
    console::info("Running final environment validation...");
    let issues = vulcanum_shared::validate::validate_environment_for_config(&config);

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

    vulcanum_shared::config::save_config(&config)?;

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
    service::enable_and_restart_worker_service()?;

    eprintln!();
    eprintln!(
        "Worker setup complete — daemon is running via {} with '{harness_name}' harness.",
        service::worker_service_label()
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
    Docker,
    None,
}

impl Backend {
    fn harness_name(&self) -> &'static str {
        match self {
            Self::Kata => "kata",
            Self::Docker => "docker",
            Self::None => "host",
        }
    }
}

fn selected_agent_backend(agent_backend: Option<AgentBackendArg>) -> AgentBackend {
    match agent_backend {
        Some(AgentBackendArg::OmpRpc) => AgentBackend::OmpRpc,
        Some(AgentBackendArg::Opencode) | None => AgentBackend::OpenCode,
    }
}
