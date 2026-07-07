use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::models::providers::model::IntegrationType;

#[derive(Debug, Clone, FromRow)]
pub struct IntegrationProvider {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub provider_type: IntegrationType,
    pub instance_url: String,
    pub api_key: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationProviderResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub provider_type: IntegrationType,
    pub instance_url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub provider_type: Option<IntegrationType>,
    pub instance_url: String,
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub provider_type: Option<IntegrationType>,
    pub instance_url: Option<String>,
    pub api_key: Option<String>,
}

impl From<IntegrationProvider> for IntegrationProviderResponse {
    fn from(provider: IntegrationProvider) -> Self {
        Self {
            id: provider.id,
            team_id: provider.team_id,
            name: provider.name,
            provider_type: provider.provider_type,
            instance_url: provider.instance_url,
            created_at: provider.created_at,
        }
    }
}
