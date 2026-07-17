use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct AppTeam {
    pub id: Uuid,
    pub name: String,
    pub primary_model_provider_key: Option<String>,
    pub primary_model_id: Option<String>,
    pub small_model_provider_key: Option<String>,
    pub small_model_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct AppWorker {
    pub id: Uuid,
    pub name: String,
    pub last_seen: Option<DateTime<Utc>>,
    pub status: String,
    pub active_jobs: i32,
    pub max_concurrent_jobs: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct TaskTracker {
    pub name: String,
    pub provider_type: String,
    pub instance_url: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct AppModelProvider {
    pub display_name: String,
    pub provider_key: String,
    pub auth_type: String,
    pub credential_fields: Vec<String>,
    pub oauth: Option<AppModelProviderOAuthStatus>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct AppModelProviderOAuthStatus {
    pub account_id: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct GithubAppInstallation {
    pub account_login: String,
}
