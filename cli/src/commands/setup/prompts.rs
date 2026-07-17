use crate::prompts::nonempty;

use crate::commands::setup::host::is_kvm_available;
use crate::commands::setup::Backend;

pub(super) fn resolve_backend(
    mode: super::InteractionMode,
    isolation: Option<crate::IsolationBackend>,
) -> anyhow::Result<Backend> {
    let backend = match isolation {
        Some(crate::IsolationBackend::Kata) => {
            if cfg!(target_os = "macos") {
                anyhow::bail!(
                    "--isolation=kata is not supported on macOS because Kata requires Linux KVM"
                );
            }
            if !is_kvm_available() {
                anyhow::bail!("--isolation=kata requires KVM, but /dev/kvm is not available");
            }
            Backend::Kata
        }
        Some(crate::IsolationBackend::Docker) => Backend::Docker,
        Some(crate::IsolationBackend::None) => Backend::None,
        None => match mode {
            super::InteractionMode::NonInteractive => Backend::Docker,
            super::InteractionMode::Interactive => {
                return prompt_backend();
            }
        },
    };

    Ok(backend)
}

fn prompt_backend() -> anyhow::Result<Backend> {
    if cfg!(target_os = "macos") {
        return prompt_macos_backend();
    }

    prompt_linux_backend()
}

fn prompt_macos_backend() -> anyhow::Result<Backend> {
    let items = vec![
        "Docker (container isolation, no sandbox)",
        "None (run directly on host)",
    ];

    let selection = dialoguer::Select::new()
        .with_prompt("Choose an isolation backend")
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => Ok(Backend::Docker),
        1 => Ok(Backend::None),
        _ => anyhow::bail!("invalid backend selection"),
    }
}

fn prompt_linux_backend() -> anyhow::Result<Backend> {
    let kvm_available = is_kvm_available();

    let items = vec![
        "Kata Containers (VM-based isolation, requires KVM)",
        "Docker (container isolation, no sandbox)",
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
        1 => Ok(Backend::Docker),
        2 => Ok(Backend::None),
        _ => anyhow::bail!("invalid backend selection"),
    }
}

pub fn prompt_code() -> anyhow::Result<String> {
    let code = dialoguer::Input::<String>::new()
        .with_prompt("Connection code")
        .validate_with(|input: &String| nonempty("Connection code", input.as_str()))
        .interact_text()?;
    Ok(code.trim().to_owned())
}
