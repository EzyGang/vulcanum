use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub code: String,
    pub worker_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub worker_id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub access_token_ttl_minutes: i64,
    pub code_ttl_minutes: i64,
    pub refresh_token_ttl_days: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PollResponse {
    pub job_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobResponse {
    pub prompt_text: String,
    pub repo_url: String,
    pub agents_md: String,
    pub external_task_ref: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitResultRequest {
    pub pr_url: String,
    pub exit_code: i32,
    pub tokens_used: i64,
    pub duration_ms: i64,
}
