use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::providers::model::{IntegrationBoard, IntegrationTask, IntegrationType};
use crate::models::work_runs::model::{WorkRunStatus, WorkRunType};

#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskProviderProject {
    pub provider_id: Uuid,
    pub provider_type: IntegrationType,
    pub workspace_id: String,
    pub external_project_id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskBoardResponse {
    pub provider_id: Uuid,
    pub provider_type: IntegrationType,
    pub board: IntegrationBoard,
    pub related_task_runs: Vec<TaskBoardTaskRelatedRuns>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskBoardTaskRelatedRuns {
    pub external_task_ref: String,
    pub runs: Vec<TaskBoardRelatedWorkRun>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskBoardRelatedWorkRun {
    pub id: Uuid,
    pub status: WorkRunStatus,
    pub work_type: WorkRunType,
    pub tokens_used: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_read_tokens: Option<i64>,
    pub cache_write_tokens: Option<i64>,
    pub model_used: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CreateTaskResponse {
    pub task: IntegrationTask,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateTaskRequest {
    pub title: String,
    #[serde(default)]
    pub body: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateTaskResponse {
    pub task: IntegrationTask,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MoveTaskRequest {
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MoveTaskResponse {
    pub task_id: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskLabelResponse {
    pub task_id: String,
    pub label_id: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskLabelDeleteResponse {
    pub label_id: String,
}
