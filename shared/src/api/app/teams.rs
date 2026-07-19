use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Default, Serialize)]
pub struct UpdateTeamModelsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_model_provider_key: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_model_id: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_model_provider_key: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_model_id: Option<Option<String>>,
}
