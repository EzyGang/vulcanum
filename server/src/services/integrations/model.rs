use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Type, Serialize, Deserialize, Default)]
#[sqlx(type_name = "integration_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IntegrationType {
    #[default]
    Kaneo,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationTask {
    pub id: String,
    pub title: String,
    pub project_id: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationColumn {
    pub id: String,
    pub name: String,
    pub slug: String,
}
