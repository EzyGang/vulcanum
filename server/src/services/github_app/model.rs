use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct GithubInstallation {
    pub id: i64,
    pub github_installation_id: i64,
    pub account_login: String,
    pub created_at: DateTime<Utc>,
}
