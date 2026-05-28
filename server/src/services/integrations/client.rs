use async_trait::async_trait;

use crate::services::kaneo::client::slugify;
use crate::services::kaneo::client::KaneoClient;

use super::errors::IntegrationError;
use super::model::{IntegrationColumn, IntegrationTask};

#[derive(Clone)]
pub enum IntegrationClient {
    Kaneo(KaneoClient),
}

impl IntegrationClient {
    pub fn new_kaneo(instance: String, api_key: String) -> Self {
        Self::Kaneo(KaneoClient::new(instance, api_key))
    }

    pub fn as_kaneo(&self) -> Option<&KaneoClient> {
        match self {
            Self::Kaneo(ref client) => Some(client),
        }
    }

    pub async fn fetch_columns(
        &self,
        project_id: &str,
    ) -> Result<Vec<IntegrationColumn>, IntegrationError> {
        match self {
            Self::Kaneo(client) => {
                let columns = client
                    .fetch_columns(project_id)
                    .await
                    .map_err(IntegrationError::from)?;
                Ok(columns
                    .iter()
                    .map(|col| {
                        let slug = col
                            .status
                            .as_deref()
                            .map(|s| s.to_owned())
                            .unwrap_or_else(|| slugify(&col.name));
                        IntegrationColumn {
                            id: col.id.clone(),
                            name: col.name.clone(),
                            slug,
                        }
                    })
                    .collect())
            }
        }
    }

    pub async fn update_task_status(
        &self,
        task_id: &str,
        new_status: &str,
    ) -> Result<(), IntegrationError> {
        match self {
            Self::Kaneo(client) => client
                .update_task_status(task_id, new_status)
                .await
                .map_err(IntegrationError::from)?,
        };
        Ok(())
    }

    pub async fn add_comment(&self, task_id: &str, content: &str) -> Result<(), IntegrationError> {
        match self {
            Self::Kaneo(client) => client
                .add_comment(task_id, content)
                .await
                .map_err(IntegrationError::from)?,
        };
        Ok(())
    }
}

#[async_trait]
impl TaskFetcher for IntegrationClient {
    async fn fetch_tasks_in_column(
        &self,
        project_id: &str,
        column_name: &str,
    ) -> Result<Vec<IntegrationTask>, IntegrationError> {
        match self {
            Self::Kaneo(client) => {
                let tasks = client
                    .fetch_tasks_in_column(project_id, column_name)
                    .await
                    .map_err(IntegrationError::from)?;
                Ok(tasks
                    .iter()
                    .map(|task| IntegrationTask {
                        id: task.id.clone(),
                        title: task.title.clone(),
                        project_id: task.project_id.clone(),
                        description: task.description.clone(),
                    })
                    .collect())
            }
        }
    }
}

#[async_trait]
pub trait TaskFetcher: Send + Sync {
    async fn fetch_tasks_in_column(
        &self,
        project_id: &str,
        column_name: &str,
    ) -> Result<Vec<IntegrationTask>, IntegrationError>;
}
