use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, sqlx::Type, Serialize)]
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
#[allow(dead_code)]
pub struct WorkRun {
    pub id: Uuid,
    pub external_task_ref: String,
    pub project_config_id: Uuid,
    pub worker_id: Option<Uuid>,
    pub status: WorkRunStatus,
    pub prompt_text: String,
    pub repo_url: String,
    pub agents_md: String,
    pub result_pr_url: Option<String>,
    pub result_exit_code: Option<i32>,
    pub tokens_used: Option<i32>,
    pub duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
