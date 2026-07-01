use uuid::Uuid;

use crate::models::providers::model::{IntegrationBoard, IntegrationTask, IntegrationType};

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
