use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProviderAuthType {
    ApiKey,
    DeviceOauth,
    None,
}

impl fmt::Display for ModelProviderAuthType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ApiKey => "api_key",
            Self::DeviceOauth => "device_oauth",
            Self::None => "none",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct AppModelProvider {
    pub id: Uuid,
    pub display_name: String,
    pub provider_key: String,
    pub auth_type: ModelProviderAuthType,
    pub credential_fields: Vec<String>,
    pub oauth: Option<AppModelProviderOAuthStatus>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct AppModelProviderOAuthStatus {
    pub account_id: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize)]
pub struct CreateModelProviderRequest {
    pub provider_key: String,
    pub display_name: String,
    pub auth_type: ModelProviderAuthType,
    pub credentials: serde_json::Value,
}

#[derive(Default, Serialize)]
pub struct UpdateModelProviderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_type: Option<ModelProviderAuthType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct StartDeviceFlowRequest {
    pub provider_key: String,
    pub device_provider: String,
    pub display_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct StartDeviceFlowResponse {
    pub attempt_id: Uuid,
    pub verification_uri: String,
    pub user_code: String,
    pub interval_seconds: i64,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PollDeviceFlowResponse {
    Pending { next_poll_at: DateTime<Utc> },
    Connected { provider: Box<AppModelProvider> },
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct ModelCatalog {
    pub providers: Vec<CatalogProvider>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct CatalogProvider {
    pub id: String,
    pub name: String,
    pub env: Vec<String>,
    pub models: Vec<CatalogModel>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct CatalogModel {
    pub id: String,
    pub name: String,
}
