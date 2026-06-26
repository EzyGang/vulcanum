use sqlx::PgPool;
use uuid::Uuid;

use crate::db::provider_configs::IntegrationProvidersRepository;
use crate::models::provider_configs::model::IntegrationProvider;
use crate::models::providers::model::{CreateIntegrationTaskInput, IntegrationColumn};
use crate::models::task_board::errors::TaskBoardError;
use crate::models::task_board::model::{
    CreateTaskRequest, CreateTaskResponse, MoveTaskResponse, TaskBoardResponse, TaskProviderProject,
};
use crate::services::providers::client::IntegrationClient;

const DEFAULT_PRIORITY: &str = "low";
const FALLBACK_STATUS: &str = "planned";

#[derive(Clone)]
pub struct TaskBoardService {
    providers_repo: IntegrationProvidersRepository,
    db: PgPool,
}

impl TaskBoardService {
    pub fn new(providers_repo: IntegrationProvidersRepository, db: PgPool) -> Self {
        Self { providers_repo, db }
    }

    pub async fn list_projects(
        &self,
        team_id: Uuid,
    ) -> Result<Vec<TaskProviderProject>, TaskBoardError> {
        let providers = self.providers_repo.list_all(&self.db, team_id).await?;
        let mut projects = Vec::new();

        for provider in providers {
            let client = IntegrationClient::from_provider(provider.clone());
            let workspaces = client.fetch_workspaces().await?;

            for workspace in workspaces {
                let workspace_projects = client.fetch_projects(&workspace.id).await?;
                projects.extend(workspace_projects.into_iter().map(|project| {
                    TaskProviderProject {
                        provider_id: provider.id,
                        provider_type: provider.provider_type,
                        workspace_id: workspace.id.clone(),
                        external_project_id: project.id,
                        name: project.name,
                        slug: project.slug,
                    }
                }));
            }
        }

        Ok(projects)
    }

    pub async fn get_board(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        external_project_id: &str,
    ) -> Result<TaskBoardResponse, TaskBoardError> {
        let provider = self.load_provider(team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(provider.clone());
        let board = client.fetch_board(external_project_id).await?;

        Ok(TaskBoardResponse {
            provider_id: provider.id,
            provider_type: provider.provider_type,
            board,
        })
    }

    pub async fn create_task(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        external_project_id: &str,
        request: CreateTaskRequest,
    ) -> Result<CreateTaskResponse, TaskBoardError> {
        let title = normalized_required(&request.title, TaskBoardError::EmptyTitle)?;
        let provider = self.load_provider(team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(provider);
        let status = match request.status.as_deref().map(str::trim) {
            Some(value) if !value.is_empty() => value.to_owned(),
            _ => {
                self.default_task_status(&client, external_project_id)
                    .await?
            }
        };
        let priority = request
            .priority
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_PRIORITY)
            .to_owned();

        let task = client
            .create_task(CreateIntegrationTaskInput {
                project_id: external_project_id.to_owned(),
                title,
                body: request.body,
                status,
                priority,
            })
            .await?;

        Ok(CreateTaskResponse { task })
    }

    pub async fn move_task(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        status: &str,
    ) -> Result<MoveTaskResponse, TaskBoardError> {
        let next_status = normalized_required(status, TaskBoardError::EmptyStatus)?;
        let provider = self.load_provider(team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(provider);

        client.update_task_status(task_id, &next_status).await?;

        Ok(MoveTaskResponse {
            task_id: task_id.to_owned(),
            status: next_status,
        })
    }

    async fn load_provider(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
    ) -> Result<IntegrationProvider, TaskBoardError> {
        self.providers_repo
            .find_by_id(&self.db, provider_id, team_id)
            .await
            .map_err(TaskBoardError::from)
    }

    async fn default_task_status(
        &self,
        client: &IntegrationClient,
        external_project_id: &str,
    ) -> Result<String, TaskBoardError> {
        let columns = client.fetch_columns(external_project_id).await?;
        Ok(default_column_status(&columns))
    }
}

pub(crate) fn default_column_status(columns: &[IntegrationColumn]) -> String {
    columns
        .iter()
        .find(|column| column.is_final != Some(true))
        .or_else(|| columns.first())
        .map(|column| column.slug.clone())
        .unwrap_or_else(|| FALLBACK_STATUS.to_owned())
}

fn normalized_required(input: &str, err: TaskBoardError) -> Result<String, TaskBoardError> {
    let value = input.trim();
    if value.is_empty() {
        return Err(err);
    }

    Ok(value.to_owned())
}
