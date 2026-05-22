use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub(crate) struct ConnectRequest<'a> {
    pub code: &'a str,
    pub worker_name: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct ConnectResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub worker_id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct RefreshRequest<'a> {
    pub refresh_token: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StatusResponse {
    pub access_token_ttl_minutes: i64,
    pub code_ttl_minutes: i64,
    pub refresh_token_ttl_days: i64,
}

#[derive(Debug, Deserialize)]
pub struct PollResponse {
    pub job_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct JobResponse {
    pub prompt_text: String,
    pub repo_url: String,
    pub agents_md: String,
    pub external_task_ref: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitResultRequest {
    pub pr_url: String,
    pub exit_code: i32,
    pub tokens_used: u64,
    pub duration_ms: u64,
}
