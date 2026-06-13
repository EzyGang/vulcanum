use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ModelProviderConfig {
    pub id: Uuid,
    pub team_id: Uuid,
    pub provider_key: String,
    pub display_name: String,
    pub credentials: serde_json::Value,
    pub advanced_options: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateModelProviderRequest {
    pub provider_key: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub credentials: serde_json::Value,
    #[serde(default)]
    pub advanced_options: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModelProviderRequest {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub credentials: Option<serde_json::Value>,
    #[serde(default)]
    pub advanced_options: Option<serde_json::Value>,
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
