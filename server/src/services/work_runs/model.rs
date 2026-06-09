use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "work_run_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum WorkRunStatus {
    Pending,
    Dispatched,
    Running,
    Completed,
    Failed,
    Stalled,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct WorkRun {
    pub id: Uuid,
    pub external_task_ref: String,
    pub project_config_id: Uuid,
    pub worker_id: Option<Uuid>,
    pub status: WorkRunStatus,
    pub prompt_text: String,
    pub repo_url: String,
    pub agents_md: String,
    pub task_title: Option<String>,
    pub task_slug: Option<String>,
    pub result_pr_url: Option<String>,
    pub result_exit_code: Option<i32>,
    pub tokens_used: Option<i64>,
    pub duration_ms: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_read_tokens: Option<i64>,
    pub cache_write_tokens: Option<i64>,
    pub model_used: Option<String>,
    pub finish_status: Option<String>,
    pub finish_summary: Option<String>,
    pub finish_blocked_reason: Option<String>,
    pub finish_next_column: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct WorkRunListItem {
    pub id: Uuid,
    pub external_task_ref: String,
    pub project_config_id: Uuid,
    pub worker_id: Option<Uuid>,
    pub worker_name: Option<String>,
    pub status: WorkRunStatus,
    pub prompt_text: String,
    pub repo_url: String,
    pub task_title: Option<String>,
    pub task_slug: Option<String>,
    pub result_pr_url: Option<String>,
    pub result_exit_code: Option<i32>,
    pub tokens_used: Option<i64>,
    pub duration_ms: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_read_tokens: Option<i64>,
    pub cache_write_tokens: Option<i64>,
    pub model_used: Option<String>,
    pub finish_status: Option<String>,
    pub finish_summary: Option<String>,
    pub finish_blocked_reason: Option<String>,
    pub finish_next_column: Option<String>,
    pub created_at: DateTime<Utc>,
}
