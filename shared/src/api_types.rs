use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::runtime::types::FinishStatus;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentBackend {
    #[default]
    OpenCode,
    OmpRpc,
}

impl AgentBackend {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenCode => "opencode",
            Self::OmpRpc => "omp_rpc",
        }
    }

    #[must_use]
    pub fn binary_name(&self) -> &'static str {
        match self {
            Self::OpenCode => "opencode",
            Self::OmpRpc => "omp",
        }
    }
}

impl std::str::FromStr for AgentBackend {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "opencode" => Ok(Self::OpenCode),
            "omp_rpc" => Ok(Self::OmpRpc),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    #[serde(default = "default_agent_backends")]
    pub agent_backends: Vec<AgentBackend>,
    #[serde(default)]
    pub isolation_backends: Vec<String>,
}

impl Default for WorkerCapabilities {
    fn default() -> Self {
        Self {
            agent_backends: default_agent_backends(),
            isolation_backends: Vec::new(),
        }
    }
}

impl WorkerCapabilities {
    #[must_use]
    pub fn supports_agent_backend(&self, backend: AgentBackend) -> bool {
        self.agent_backends.contains(&backend)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpenCodeProviderConfig {
    pub options: HashMap<String, String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum AgentConfigPayload {
    OpenCode {
        providers: HashMap<String, OpenCodeProviderConfig>,
        model: Option<String>,
        small_model: Option<String>,
        auth_content: Option<String>,
    },
    OmpRpc {
        config_yml: Option<String>,
    },
}

impl AgentConfigPayload {
    #[must_use]
    pub fn backend(&self) -> AgentBackend {
        match self {
            Self::OpenCode { .. } => AgentBackend::OpenCode,
            Self::OmpRpc { .. } => AgentBackend::OmpRpc,
        }
    }
}

fn default_agent_backends() -> Vec<AgentBackend> {
    vec![AgentBackend::OpenCode]
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkRunType {
    Implementation,
    PullRequestReview,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub code: String,
    pub worker_name: String,
    pub max_concurrent_jobs: Option<i32>,
    #[serde(default)]
    pub capabilities: WorkerCapabilities,
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
    pub work_type: WorkRunType,
    pub prompt_text: String,
    pub repos: Vec<JobRepo>,
    pub agents_md: String,
    pub agent_backend: AgentBackend,
    pub agent_config: AgentConfigPayload,
    pub model_provider_env: HashMap<String, String>,
    pub external_task_ref: String,
    pub provider_instance_url: String,
    pub provider_api_key: String,
    pub external_project_id: String,
    pub external_workspace_id: String,
    pub max_turns: i32,
    pub github_token: Option<String>,
    #[serde(default)]
    pub github_token_expires_at: Option<DateTime<Utc>>,
    pub pr_urls: Vec<String>,
    pub review_target_pr_url: Option<String>,
    pub review_target_repo_full_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshGithubTokenResponse {
    pub github_token: Option<String>,
    pub github_token_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct JobRepo {
    pub full_name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResultRequest {
    #[serde(default)]
    pub pr_urls: Vec<String>,
    pub exit_code: i32,
    pub tokens_used: i64,
    pub duration_ms: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub model_used: Option<String>,
    pub finish_status: Option<FinishStatus>,
    pub result_summary: Option<String>,
    // review_url, review_body, and review_already_exists are kept on the wire
    // (not stored in `work_runs`) so `record_review_result` can persist them
    // into the `work_run_reviews` table on the server side.
    pub review_url: Option<String>,
    pub review_body: Option<String>,
    #[serde(default)]
    pub review_already_exists: bool,
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
