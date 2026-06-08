use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::runtime::types::FinishStatus;

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
    pub max_concurrent_jobs: i32,
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
pub struct AckRequest {}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobResponse {
    pub prompt_text: String,
    pub repo_url: String,
    pub agents_md: String,
    pub opencode_config: String,
    pub external_task_ref: String,
    pub provider_instance_url: String,
    pub provider_api_key: String,
    pub external_project_id: String,
    pub external_workspace_id: String,
    pub max_turns: i32,
    pub github_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitResultRequest {
    pub pr_url: String,
    pub exit_code: i32,
    pub tokens_used: i64,
    pub duration_ms: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub model_used: Option<String>,
    pub finish_status: Option<FinishStatus>,
    pub finish_summary: Option<String>,
    pub finish_blocked_reason: Option<String>,
    pub finish_next_column: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkDeleteRunsRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkDeleteRunsResponse {
    pub deleted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireEvent {
    pub sequence: u64,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppendEventsRequest {
    pub events: Vec<WireEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppendEventsResponse {
    pub accepted: u64,
    pub should_cancel: bool,
}

#[derive(Debug, Serialize)]
pub struct ListEventsResponse {
    pub events: Vec<WireEvent>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub after_occurred_at: Option<chrono::DateTime<chrono::Utc>>,
    pub after_sequence: Option<u64>,
    pub limit: Option<i64>,
}
