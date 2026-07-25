use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct GithubInstallation {
    pub id: i64,
    pub team_id: Uuid,
    pub github_installation_id: i64,
    pub account_login: String,
    pub installed_by_user_id: Option<String>,
    pub review_identity_user_id: Option<String>,
    pub review_identity_login: Option<String>,
    pub created_at: DateTime<Utc>,
}
