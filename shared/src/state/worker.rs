use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerState {
    pub worker_id: Uuid,
    pub instance_url: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub max_concurrent_jobs: i32,
}

pub fn state_path() -> anyhow::Result<PathBuf> {
    crate::state::paths::state_path()
}

pub fn load_state() -> anyhow::Result<Option<WorkerState>> {
    let path = state_path()?;
    crate::state::file::load_json(&path)
}

pub fn save_state(state: &WorkerState) -> anyhow::Result<()> {
    let path = state_path()?;
    crate::state::file::save_json(&path, state)
}
