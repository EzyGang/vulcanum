use vulcanum_shared::validate::{validate_environment_for_backend, Severity};
use vulcanum_shared::worker_state;

use crate::console;

mod docker;
pub(crate) mod gvisor;
pub(crate) mod image;
mod kata;
pub(crate) mod systemd;
mod utils;

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
        Backend::None => {
            console::info("Skipping container runtime installation (host mode).");
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
enum Backend {
    Kata,
    Gvisor,
    None,
}

impl Backend {
    fn harness_name(&self) -> &'static str {
        match self {
            Self::Kata => "kata",
            Self::Gvisor => "gvisor",
            Self::None => "host",
        }
    }
}

fn resolve_backend(
    mode: InteractionMode,
    isolation: Option<crate::IsolationBackend>,
) -> anyhow::Result<Backend> {
    let backend = match isolation {
        Some(crate::IsolationBackend::Kata) => {
            if !utils::is_kvm_available() {
                anyhow::bail!("--isolation=kata requires KVM, but /dev/kvm is not available");
            }
            Backend::Kata
        }
        Some(crate::IsolationBackend::Gvisor) => Backend::Gvisor,
        Some(crate::IsolationBackend::None) => Backend::None,
        None => match mode {
            InteractionMode::NonInteractive => {
                anyhow::bail!(
                    "--isolation is required in non-interactive mode (kata, gvisor, or none)"
                );
            }
            InteractionMode::Interactive => {
                return prompt_backend();
            }
        },
    };

    Ok(backend)
}

fn prompt_backend() -> anyhow::Result<Backend> {
    let kvm_available = utils::is_kvm_available();

    let items = vec![
        "Kata Containers (VM-based isolation, requires KVM)",
        "gVisor (container sandboxing)",
        "None (run directly on host)",
    ];

    let default = if kvm_available { 0 } else { 1 };

    let selection = dialoguer::Select::new()
        .with_prompt("Choose an isolation backend")
        .items(&items)
        .default(default)
        .interact()?;

    match selection {
        0 => {
            if !kvm_available {
                anyhow::bail!("Kata Containers requires KVM, but /dev/kvm is not available");
            }
            Ok(Backend::Kata)
        }
        1 => Ok(Backend::Gvisor),
        2 => Ok(Backend::None),
        _ => anyhow::bail!("invalid backend selection"),
    }
}

async fn verify_connection() -> anyhow::Result<()> {
    use vulcanum_shared::client::ApiClient;
    use vulcanum_shared::worker_state;

    let state =
        worker_state::load_state()?.ok_or_else(|| anyhow::anyhow!("no worker state found"))?;
    let client = ApiClient::new(&state.instance_url);
    client.status().await?;
    Ok(())
}

async fn connect_worker(code: Option<String>, instance: Option<String>) -> anyhow::Result<()> {
    use vulcanum_shared::client::{probe_url_with_scheme_fallback, ApiClient};
    use vulcanum_shared::worker_state::{save_state, WorkerState};

    let raw_instance = match instance {
        Some(url) => url,
        None => prompt_instance_url()?,
    };

    console::info("Probing instance URL...");
    let (resolved_url, _) = probe_url_with_scheme_fallback(&raw_instance).await?;
    if resolved_url != raw_instance.trim_end_matches('/') {
        console::info(&format!("Using {} (scheme fallback)", resolved_url));
    }

    let code = match code {
        Some(c) => c,
        None => prompt_code()?,
    };

    let worker_name = hostname::get()
        .ok()
        .and_then(|h| h.to_str().map(|s| s.to_owned()))
        .unwrap_or_else(|| "unnamed-worker".to_owned());

    let client = ApiClient::new(&resolved_url);

    let resp = client.connect(&code, &worker_name).await?;

    let state = WorkerState {
        worker_id: resp.worker_id,
        instance_url: resolved_url,
        access_token: resp.access_token,
        refresh_token: resp.refresh_token,
        expires_at: resp.expires_at,
    };

    save_state(&state)?;

    tracing::info!(
        worker_id = resp.worker_id.to_string().as_str(),
        worker_name = resp.name.as_str(),
        "connected as worker '{}' (id: {}, token expires: {})",
        resp.name,
        resp.worker_id,
        resp.expires_at
    );

    if systemd::is_unit_installed() {
        tracing::info!("restarting systemd service after connect");
        systemd::enable_and_restart_service()?;
    }

    Ok(())
}

fn nonempty(field: &str, input: &str) -> Result<(), String> {
    if input.trim().is_empty() {
        return Err(format!("{field} is required"));
    }
    Ok(())
}

fn normalize_instance_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_owned()
    } else {
        format!("https://{trimmed}")
    }
}

fn prompt_instance_url() -> anyhow::Result<String> {
    let url = dialoguer::Input::<String>::new()
        .with_prompt("Instance URL")
        .validate_with(|input: &String| {
            let normalized = normalize_instance_url(input);
            nonempty("Instance URL", input)?;
            match url::Url::parse(&normalized) {
                Ok(_) => Ok(()),
                Err(_) => Err("Please enter a valid URL".to_owned()),
            }
        })
        .interact_text()?;
    Ok(normalize_instance_url(&url))
}

fn prompt_code() -> anyhow::Result<String> {
    let code = dialoguer::Input::<String>::new()
        .with_prompt("Connection code")
        .validate_with(|input: &String| nonempty("Connection code", input))
        .interact_text()?;
    Ok(code.trim().to_owned())
}
