use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
#[allow(dead_code)]
pub struct WorkRun {
    pub id: Uuid,
    pub external_task_ref: String,
    pub project_config_id: Uuid,
    pub worker_id: Option<Uuid>,
    pub status: String,
    pub prompt_text: String,
    pub result_pr_url: Option<String>,
    pub result_exit_code: Option<i32>,
    pub tokens_used: Option<i32>,
    pub duration_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
