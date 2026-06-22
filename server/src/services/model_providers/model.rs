use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub const AUTH_TYPE_API_KEY: &str = "api_key";
pub const AUTH_TYPE_CHATGPT_OAUTH: &str = "chatgpt_oauth";
pub const OPENAI_PROVIDER_KEY: &str = "openai";

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ModelProviderConfig {
    pub id: Uuid,
    pub team_id: Uuid,
    pub provider_key: String,
    pub auth_type: String,
    pub display_name: String,
    pub credentials: serde_json::Value,
    #[serde(skip_serializing)]
    pub oauth_credentials: Option<serde_json::Value>,
    pub oauth_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateModelProviderRequest {
    pub provider_key: String,
    #[serde(default = "default_auth_type")]
    pub auth_type: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub credentials: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModelProviderRequest {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub credentials: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCredentials {
    pub access: String,
    pub refresh: String,
    pub expires: i64,
    #[serde(default)]
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthMetadata {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ChatGptAuthAttempt {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: String,
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub interval_seconds: i32,
    pub expires_at: DateTime<Utc>,
    pub status: String,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatGptAuthStartResponse {
    pub attempt_id: Uuid,
    pub verification_uri: String,
    pub user_code: String,
    pub expires_at: DateTime<Utc>,
    pub poll_interval_seconds: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatGptAuthStatusResponse {
    pub status: String,
    pub error: Option<String>,
    pub provider: Option<ModelProviderConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StartChatGptAuthRequest {
    #[serde(default)]
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogResponse {
    pub providers: Vec<CatalogProvider>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogProvider {
    pub id: String,
    pub name: String,
    pub doc: String,
    pub env: Vec<String>,
    pub models: Vec<CatalogModel>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogModel {
    pub id: String,
    pub name: String,
    pub status: Option<String>,
    pub context_limit: Option<i64>,
    pub output_limit: Option<i64>,
    pub input_cost: Option<f64>,
    pub output_cost: Option<f64>,
    pub attachment: bool,
    pub reasoning: bool,
    pub tool_call: bool,
    pub structured_output: bool,
}

fn default_auth_type() -> String {
    AUTH_TYPE_API_KEY.to_owned()
}
