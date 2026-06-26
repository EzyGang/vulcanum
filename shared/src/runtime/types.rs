use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishStatus {
    Completed,
    Failed,
    Blocked,
}

impl fmt::Display for FinishStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FinishStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceLimits {
    pub max_duration_secs: u64,
    pub vcpu_count: u64,
    pub memory_mib: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_duration_secs: 21_600,
            vcpu_count: 2,
            memory_mib: 1_024,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl SessionStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone)]
pub struct SessionExport {
    pub exit_code: i32,
    pub tokens_used: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub duration_ms: u64,
    pub model_used: Option<String>,
    pub failure_payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub event_type: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct IsolatedEnvironment {
    pub workdir: PathBuf,
    pub workspace_dir: PathBuf,
    pub repos: Vec<WorkspaceRepo>,
    pub container_name: Option<String>,
    pub secrets: HashMap<String, String>,
    pub env_vars: HashMap<String, String>,
    pub runtime: Option<&'static str>,
    pub image: Option<String>,
    pub server_host_port: Option<u16>,
    pub limits: ResourceLimits,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkspaceRepo {
    pub full_name: String,
    pub url: String,
    pub relative_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FinishRunArtifact {
    pub status: FinishStatus,
    #[serde(default)]
    pub pr_urls: Vec<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub review_url: Option<String>,
    #[serde(default)]
    pub review_body: Option<String>,
    #[serde(default)]
    pub review_already_exists: bool,
}
