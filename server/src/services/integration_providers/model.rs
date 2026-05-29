use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::services::integrations::model::IntegrationType;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct IntegrationProvider {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "providerType")]
    pub provider_type: IntegrationType,
    pub instance_url: String,
    pub api_key: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    #[serde(rename = "instanceUrl")]
    pub instance_url: String,
    #[serde(rename = "apiKey")]
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    #[serde(rename = "instanceUrl")]
    pub instance_url: Option<String>,
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
}
