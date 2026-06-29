use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Type, Serialize, Deserialize, Default)]
#[sqlx(type_name = "integration_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IntegrationType {
    #[default]
    Kaneo,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct IntegrationLabel {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationTask {
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
    pub labels: Vec<IntegrationLabel>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationProject {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub workspace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationColumn {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub is_final: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationBoardColumn {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub is_final: Option<bool>,
    pub tasks: Vec<IntegrationTask>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationBoard {
    pub project: IntegrationProject,
    pub columns: Vec<IntegrationBoardColumn>,
    pub labels: Vec<IntegrationLabel>,
}

#[derive(Debug, Clone)]
pub struct CreateIntegrationTaskInput {
    pub project_id: String,
    pub title: String,
    pub body: String,
    pub status: String,
    pub priority: String,
}

#[derive(Debug, Clone)]
pub struct UpdateIntegrationTaskInput {
    pub task_id: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct CreateIntegrationLabelInput {
    pub workspace_id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct UpdateIntegrationLabelInput {
    pub label_id: String,
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationWorkspace {
    pub id: String,
    pub name: String,
}
