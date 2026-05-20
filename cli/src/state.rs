use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerState {
    pub worker_id: Uuid,
    pub instance_url: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
}

pub fn state_path() -> anyhow::Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("failed to find config directory")?
        .join("vulcanum");
    Ok(dir.join("worker.json"))
}

pub fn load_state() -> anyhow::Result<Option<WorkerState>> {
    let path = state_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read state from {}", path.display()))?;
    let state: WorkerState = serde_json::from_str(&data)
        .with_context(|| format!("failed to parse state from {}", path.display()))?;
    Ok(Some(state))
}

pub fn save_state(state: &WorkerState) -> anyhow::Result<()> {
    let path = state_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config dir {}", parent.display()))?;
    }
    let data = serde_json::to_string_pretty(state).context("failed to serialize state")?;
    std::fs::write(&path, data)
        .with_context(|| format!("failed to write state to {}", path.display()))?;
    Ok(())
}
