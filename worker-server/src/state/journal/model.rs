use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum JournalStatus {
    Running,
    Completed,
    Failed,
    Lost,
    Submitted,
}

impl JournalStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Lost => "lost",
            Self::Submitted => "submitted",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "lost" => Some(Self::Lost),
            "submitted" => Some(Self::Submitted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
#[expect(dead_code)]
pub struct JournalEntry {
    pub job_id: Uuid,
    pub workdir: String,
    pub container_name: Option<String>,
    pub harness_type: String,
    pub status: JournalStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub tokens_used: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_read_tokens: Option<i64>,
    pub cache_write_tokens: Option<i64>,
    pub pr_url: Option<String>,
    pub duration_ms: Option<i64>,
    pub review_url: Option<String>,
    pub review_body: Option<String>,
    pub review_already_exists: bool,
    pub error_message: Option<String>,
    pub turn_count: Option<i32>,
    pub session_id: Option<String>,
    pub max_turns: Option<i32>,
    pub host_pid: Option<i64>,
    pub host_port: Option<i64>,
    pub agent_backend: Option<String>,
    pub agent_session_path: Option<String>,
    pub agent_config_dir: Option<String>,
    pub agent_state_dir: Option<String>,
    pub agent_transport: Option<String>,
    pub agent_pid: Option<i64>,
}

pub struct JournalResultUpdate<'a> {
    pub job_id: Uuid,
    pub status: JournalStatus,
    pub exit_code: i32,
    pub tokens_used: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub pr_url: Option<&'a str>,
    pub duration_ms: i64,
    pub review_url: Option<&'a str>,
    pub review_body: Option<&'a str>,
    pub review_already_exists: bool,
}

pub struct JournalInsert<'a> {
    pub job_id: Uuid,
    pub workdir: &'a str,
    pub container_name: Option<&'a str>,
    pub harness_type: &'a str,
    pub started_at: DateTime<Utc>,
    pub max_turns: i32,
    pub agent_backend: &'a str,
}

pub(super) fn journal_status_from_str(s: &str) -> Option<JournalStatus> {
    JournalStatus::from_str(s)
}
