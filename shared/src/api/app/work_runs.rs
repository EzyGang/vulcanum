use std::fmt;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkRunStatus {
    Pending,
    Dispatched,
    Running,
    Completed,
    Failed,
    Stalled,
}

impl fmt::Display for WorkRunStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Pending => "pending",
            Self::Dispatched => "dispatched",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Stalled => "stalled",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkRunType {
    Implementation,
    PullRequestReview,
}

impl fmt::Display for WorkRunType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Implementation => "implementation",
            Self::PullRequestReview => "review",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct WorkRunListItem {
    pub id: Uuid,
    pub external_task_ref: String,
    pub task_title: Option<String>,
    pub status: WorkRunStatus,
    pub work_type: WorkRunType,
    pub tokens_used: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_read_tokens: Option<i64>,
    pub cache_write_tokens: Option<i64>,
    pub model_used: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
}
