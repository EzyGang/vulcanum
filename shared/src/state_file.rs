use std::io::Write;
use std::path::Path;

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::Serialize;
#[cfg(all(test, unix))]
pub(crate) static HOME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn load_json<T>(path: &Path) -> anyhow::Result<Option<T>>
where
    T: DeserializeOwned,
{
    if !path.exists() {
        return Ok(None);
    }

    let data = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read state from {}", path.display()))?;
    let value = serde_json::from_str(&data)
        .with_context(|| format!("failed to parse state from {}", path.display()))?;
    Ok(Some(value))
}

pub(crate) fn save_json<T>(path: &Path, value: &T) -> anyhow::Result<()>
where
    T: Serialize,
{
    path.parent()
        .map(|parent| {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create config dir {}", parent.display()))
        })
        .transpose()?;

    let data = serde_json::to_string_pretty(value).context("failed to serialize state")?;
    let mut file = create_restricted_file(path)?;
    file.write_all(data.as_bytes())
        .with_context(|| format!("failed to write state to {}", path.display()))?;
    Ok(())
}

#[cfg(unix)]
fn create_restricted_file(path: &Path) -> anyhow::Result<std::fs::File> {
    use std::fs::OpenOptions;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .with_context(|| format!("failed to open {} for writing", path.display()))?;

    file.set_permissions(std::fs::Permissions::from_mode(0o600))
        .with_context(|| format!("failed to restrict permissions on {}", path.display()))?;

    Ok(file)
}

#[cfg(not(unix))]
fn create_restricted_file(path: &Path) -> anyhow::Result<std::fs::File> {
    std::fs::File::create(path)
        .with_context(|| format!("failed to open {} for writing", path.display()))
}
