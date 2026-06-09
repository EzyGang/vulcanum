#[cfg(test)]
mod client_tests;

use async_trait::async_trait;

use crate::services::providers::kaneo::client::KaneoClient;

use super::errors::IntegrationError;
use super::model::{IntegrationColumn, IntegrationProject, IntegrationTask, IntegrationWorkspace};

#[derive(Clone)]
pub enum IntegrationClient {
    Kaneo(KaneoClient),
}

impl IntegrationClient {
    pub fn new_kaneo(instance: String, api_key: String) -> Self {
        Self::Kaneo(KaneoClient::new(instance, api_key))
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
                            .filter(|status| !status.is_empty())
                            .map(str::to_owned)
                            .unwrap_or_else(|| column_name_to_slug(&col.name));
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

    pub async fn lookup_project(
        &self,
        project_id: &str,
    ) -> Result<IntegrationProject, IntegrationError> {
        match self {
            Self::Kaneo(client) => {
                let project = client
                    .lookup_project(project_id)
                    .await
                    .map_err(IntegrationError::from)?;
                Ok(IntegrationProject {
                    id: project.id,
                    name: project.name,
                    slug: project.slug,
                })
            }
        }
    }

    pub fn instance_and_key(&self) -> (&str, &str) {
        match self {
            Self::Kaneo(client) => (&client.instance, &client.api_key),
        }
    }

    pub async fn fetch_workspaces(&self) -> Result<Vec<IntegrationWorkspace>, IntegrationError> {
        match self {
            Self::Kaneo(client) => {
                let workspaces = client
                    .fetch_workspaces()
                    .await
                    .map_err(IntegrationError::from)?;
                Ok(workspaces
                    .into_iter()
                    .map(|w| IntegrationWorkspace {
                        id: w.id,
                        name: w.name,
                    })
                    .collect())
            }
        }
    }

    pub async fn fetch_projects(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<IntegrationProject>, IntegrationError> {
        match self {
            Self::Kaneo(client) => {
                let projects = client
                    .fetch_projects(workspace_id)
                    .await
                    .map_err(IntegrationError::from)?;
                Ok(projects
                    .into_iter()
                    .map(|p| IntegrationProject {
                        id: p.id,
                        name: p.name,
                        slug: p.slug,
                    })
                    .collect())
            }
        }
    }
}

pub(crate) fn column_name_to_slug(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
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
                let (tasks, project_slug) = client
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
                        number: task.number,
                        project_slug: Some(project_slug.clone()),
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
