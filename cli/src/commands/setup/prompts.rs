use super::utils::is_kvm_available;
use super::Backend;

fn normalize_instance_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_owned()
    } else {
        format!("https://{trimmed}")
    }
}

fn nonempty(field: &str, input: &str) -> Result<(), String> {
    if input.trim().is_empty() {
        return Err(format!("{field} is required"));
    }
    Ok(())
}

pub(super) fn resolve_backend(
    mode: super::InteractionMode,
    isolation: Option<crate::IsolationBackend>,
) -> anyhow::Result<Backend> {
    let backend = match isolation {
        Some(crate::IsolationBackend::Kata) => {
            if !is_kvm_available() {
                anyhow::bail!("--isolation=kata requires KVM, but /dev/kvm is not available");
            }
            Backend::Kata
        }
        Some(crate::IsolationBackend::Gvisor) => Backend::Gvisor,
        Some(crate::IsolationBackend::None) => Backend::None,
        None => match mode {
            super::InteractionMode::NonInteractive => {
                anyhow::bail!(
                    "--isolation is required in non-interactive mode (kata, gvisor, or none)"
                );
            }
            super::InteractionMode::Interactive => {
                return prompt_backend();
            }
        },
    };

    Ok(backend)
}

fn prompt_backend() -> anyhow::Result<Backend> {
    let kvm_available = is_kvm_available();

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

pub fn prompt_instance_url() -> anyhow::Result<String> {
    let url = dialoguer::Input::<String>::new()
        .with_prompt("Instance URL")
        .validate_with(|input: &String| {
            let normalized = normalize_instance_url(input);
            nonempty("Instance URL", input.as_str())?;
            match url::Url::parse(&normalized) {
                Ok(_) => Ok(()),
                Err(_) => Err("Please enter a valid URL".to_owned()),
            }
        })
        .interact_text()?;
    Ok(normalize_instance_url(&url))
}

pub fn prompt_code() -> anyhow::Result<String> {
    let code = dialoguer::Input::<String>::new()
        .with_prompt("Connection code")
        .validate_with(|input: &String| nonempty("Connection code", input.as_str()))
        .interact_text()?;
    Ok(code.trim().to_owned())
}
