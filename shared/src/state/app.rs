use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AppSession {
    pub instance_url: String,
    pub access_token: String,
    pub refresh_token: String,
    pub refresh_expires_at: DateTime<Utc>,
    #[serde(default)]
    pub team_id: Option<Uuid>,
}

pub fn load_state() -> anyhow::Result<Option<AppSession>> {
    let path = crate::state::paths::app_state_path()?;
    crate::state::file::load_json(&path)
}

pub fn save_state(state: &AppSession) -> anyhow::Result<()> {
    let path = crate::state::paths::app_state_path()?;
    crate::state::file::save_json(&path, state)
}
