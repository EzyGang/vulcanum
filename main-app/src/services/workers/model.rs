use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::Type, Serialize)]
#[sqlx(type_name = "worker_status", rename_all = "snake_case")]
pub enum WorkerStatus {
    Idle,
    Busy,
    Disconnected,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Worker {
    pub id: Uuid,
    pub name: String,
    pub refresh_token_hash: String,
    pub refresh_expires_at: DateTime<Utc>,
    pub last_seen: Option<DateTime<Utc>>,
    pub status: WorkerStatus,
    pub capabilities: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ConnectRequest {
    pub code: String,
    pub worker_name: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct ConnectResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub worker_id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CodeResponse {
    pub code: String,
    pub expires_at: DateTime<Utc>,
}
