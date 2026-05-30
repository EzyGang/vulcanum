use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub use vulcanum_shared::api_types::{
    ConnectRequest, ConnectResponse, RefreshRequest, RefreshResponse,
};

pub const CODE_TTL_MINUTES: i64 = 10;
pub const ACCESS_TOKEN_TTL_MINUTES: i64 = 15;
pub const REFRESH_TOKEN_TTL_DAYS: i64 = 30;
pub const TOKEN_LENGTH: usize = 64;
pub const CODE_LENGTH: usize = 16;
pub const DEFAULT_MAX_CONCURRENT_JOBS: i32 = 3;
pub const DEFAULT_UNHEALTHY_THRESHOLD: i32 = 3;

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "worker_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    Idle,
    Busy,
    Disconnected,
    Unhealthy,
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
    pub active_jobs: i32,
    pub max_concurrent_jobs: i32,
    pub consecutive_errors: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkerResponse {
    pub id: Uuid,
    pub name: String,
    pub last_seen: Option<DateTime<Utc>>,
    pub status: WorkerStatus,
    pub capabilities: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub active_jobs: i32,
    pub max_concurrent_jobs: i32,
    pub consecutive_errors: i32,
}

impl From<Worker> for WorkerResponse {
    fn from(w: Worker) -> Self {
        Self {
            id: w.id,
            name: w.name,
            last_seen: w.last_seen,
            status: w.status,
            capabilities: w.capabilities,
            created_at: w.created_at,
            active_jobs: w.active_jobs,
            max_concurrent_jobs: w.max_concurrent_jobs,
            consecutive_errors: w.consecutive_errors,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkerStatusRequest {
    pub status: WorkerStatus,
}

#[derive(Debug, Serialize)]
pub struct CodeResponse {
    pub code: String,
    pub expires_at: DateTime<Utc>,
}
