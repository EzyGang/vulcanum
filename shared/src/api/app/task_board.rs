use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct TaskBoardResponse {
    pub provider_id: Uuid,
    pub provider_type: String,
    pub board: TaskBoard,
    pub project_usage: TaskBoardProjectUsage,
    pub task_augmentations: Vec<TaskAugmentation>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct TaskBoard {
    pub project: TaskBoardProject,
    pub columns: Vec<TaskBoardColumn>,
    pub labels: Vec<TaskLabel>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct TaskBoardProject {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct TaskBoardColumn {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub is_final: Option<bool>,
    pub tasks: Vec<Task>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub project_id: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub number: Option<i64>,
    pub project_slug: Option<String>,
    pub assignee_name: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub labels: Vec<TaskLabel>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct TaskLabel {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
pub struct TaskBoardUsageCounters {
    pub tokens_used: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub finished_runs_count: i64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
pub struct TaskBoardProjectUsage {
    pub total: TaskBoardUsageCounters,
    pub this_week: TaskBoardUsageCounters,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct TaskAugmentation {
    pub external_task_ref: String,
    pub tokens_used: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub finished_runs_count: i64,
    pub pr_urls: Vec<String>,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub body: String,
    pub status: Option<String>,
    pub priority: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct CreateTaskResponse {
    pub task: Task,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UpdateTaskRequest {
    pub title: String,
    pub body: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct UpdateTaskResponse {
    pub task: Task,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MoveTaskRequest {
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct MoveTaskResponse {
    pub task_id: String,
    pub status: String,
}
