#[cfg(test)]
mod client_tests;

use async_trait::async_trait;
use kaneo_cli::api::types::{
    BoardColumn as KaneoBoardColumn, BoardResponse as KaneoBoardResponse, Task as KaneoTask,
};

use crate::models::provider_configs::model::IntegrationProvider;
use crate::models::providers::errors::IntegrationError;
use crate::models::providers::model::{
    CreateIntegrationTaskInput, IntegrationBoard, IntegrationBoardColumn, IntegrationColumn,
    IntegrationProject, IntegrationTask, IntegrationType, IntegrationWorkspace,
};
use crate::services::providers::kaneo::client::KaneoClient;

#[derive(Clone)]
pub enum IntegrationClient {
    Kaneo(KaneoClient),
}

impl IntegrationClient {
    #[must_use]
    pub fn from_provider(provider: &IntegrationProvider) -> Self {
        match provider.provider_type {
            IntegrationType::Kaneo => {
                Self::new_kaneo(provider.instance_url.clone(), provider.api_key.clone())
            }
        }
    }

    #[must_use]
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
                            is_final: col.is_final,
                        }
                    })
                    .collect())
            }
        }
    }

    pub async fn fetch_board(
        &self,
        project_id: &str,
    ) -> Result<IntegrationBoard, IntegrationError> {
        match self {
            Self::Kaneo(client) => {
                let board = client
                    .fetch_board(project_id)
                    .await
                    .map_err(IntegrationError::from)?;
                Ok(kaneo_board_to_integration(board))
            }
        }
    }

    pub async fn create_task(
        &self,
        input: CreateIntegrationTaskInput,
    ) -> Result<IntegrationTask, IntegrationError> {
        match self {
            Self::Kaneo(client) => {
                let task = client
                    .create_task(
                        &input.project_id,
                        &input.title,
                        &input.body,
                        &input.status,
                        &input.priority,
                    )
                    .await
                    .map_err(IntegrationError::from)?;
                Ok(kaneo_task_to_integration(&task, None))
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

    pub async fn update_task_description(
        &self,
        task_id: &str,
        description: &str,
    ) -> Result<(), IntegrationError> {
        match self {
            Self::Kaneo(client) => client
                .update_task_description(task_id, description)
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

#[must_use]
pub(crate) fn column_name_to_slug(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
}

#[must_use]
pub(crate) fn kaneo_task_to_integration(
    task: &KaneoTask,
    project_slug: Option<&str>,
) -> IntegrationTask {
    IntegrationTask {
        id: task.id.clone(),
        title: task.title.clone(),
        project_id: task.project_id.clone(),
        description: task.description.clone(),
        status: task.status.clone(),
        priority: task.priority.clone(),
        number: task.number,
        project_slug: project_slug.map(str::to_owned),
        assignee_name: task.assignee_name.clone(),
        created_at: task.created_at.clone(),
        updated_at: task.updated_at.clone(),
    }
}

#[must_use]
pub(crate) fn kaneo_column_slug(name: &str, status: Option<&str>) -> String {
    status
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| column_name_to_slug(name))
}

#[must_use]
pub(crate) fn kaneo_board_column_to_integration(
    column: &KaneoBoardColumn,
    project_slug: &str,
) -> IntegrationBoardColumn {
    IntegrationBoardColumn {
        id: column.id.clone(),
        name: column.name.clone(),
        slug: kaneo_column_slug(&column.name, column.status.as_deref()),
        is_final: column.is_final,
        tasks: column
            .tasks
            .iter()
            .map(|task| kaneo_task_to_integration(task, Some(project_slug)))
            .collect(),
    }
}

#[must_use]
pub(crate) fn kaneo_board_to_integration(board: KaneoBoardResponse) -> IntegrationBoard {
    let data = board.data;
    let project = IntegrationProject {
        id: data.id,
        name: data.name,
        slug: data.slug,
    };

    let mut columns = data
        .columns
        .iter()
        .map(|column| kaneo_board_column_to_integration(column, &project.slug))
        .collect::<Vec<IntegrationBoardColumn>>();

    if !data.planned_tasks.is_empty() {
        columns.push(IntegrationBoardColumn {
            id: "planned".to_owned(),
            name: "Planned".to_owned(),
            slug: "planned".to_owned(),
            is_final: Some(false),
            tasks: data
                .planned_tasks
                .iter()
                .map(|task| kaneo_task_to_integration(task, Some(&project.slug)))
                .collect(),
        });
    }

    if !data.archived_tasks.is_empty() {
        columns.push(IntegrationBoardColumn {
            id: "archived".to_owned(),
            name: "Archived".to_owned(),
            slug: "archived".to_owned(),
            is_final: Some(true),
            tasks: data
                .archived_tasks
                .iter()
                .map(|task| kaneo_task_to_integration(task, Some(&project.slug)))
                .collect(),
        });
    }

    IntegrationBoard { project, columns }
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
                    .map(|task| kaneo_task_to_integration(task, Some(project_slug.as_str())))
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
