use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::services::model_providers::auth::credentials::ModelProviderAuthType;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ModelProviderConfig {
    pub id: Uuid,
    pub team_id: Uuid,
    pub provider_key: String,
    pub display_name: String,
    pub credentials: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelProviderResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub provider_key: String,
    pub display_name: String,
    pub auth_type: ModelProviderAuthType,
    pub credential_fields: Vec<String>,
    pub oauth: Option<ModelProviderOAuthStatus>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelProviderOAuthStatus {
    pub provider: String,
    pub account_id: Option<String>,
    pub email: Option<String>,
    pub expires: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateModelProviderRequest {
    pub provider_key: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default = "default_api_key_auth_type")]
    pub auth_type: ModelProviderAuthType,
    #[serde(default)]
    pub credentials: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateModelProviderRequest {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub auth_type: Option<ModelProviderAuthType>,
    #[serde(default)]
    pub credentials: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct StartDeviceFlowRequest {
    pub provider_key: String,
    pub device_provider: String,
    #[serde(default)]
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct StartDeviceFlowResponse {
    pub attempt_id: Uuid,
    pub verification_uri: String,
    pub user_code: String,
    pub interval_seconds: i64,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PollDeviceFlowResponse {
    Pending {
        next_poll_at: DateTime<Utc>,
    },
    Connected {
        provider: Box<ModelProviderResponse>,
    },
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
    pub opencode_chatgpt_compatible: bool,
}

fn default_api_key_auth_type() -> ModelProviderAuthType {
    ModelProviderAuthType::ApiKey
}
