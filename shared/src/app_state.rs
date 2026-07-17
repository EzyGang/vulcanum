use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AppSession {
    pub instance_url: String,
    pub access_token: String,
    pub refresh_token: String,
    pub refresh_expires_at: DateTime<Utc>,
}

pub fn load_state() -> anyhow::Result<Option<AppSession>> {
    let path = crate::paths::app_state_path()?;
    crate::state_file::load_json(&path)
}

pub fn save_state(state: &AppSession) -> anyhow::Result<()> {
    let path = crate::paths::app_state_path()?;
    crate::state_file::save_json(&path, state)
}
