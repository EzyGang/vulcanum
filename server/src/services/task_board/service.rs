use crate::db::queryer::Queryer;
use uuid::Uuid;

use crate::db::project_configs::ProjectConfigsRepository;
use crate::db::provider_configs::IntegrationProvidersRepository;
use crate::models::project_configs::model::ProjectConfig;
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
    project_configs_repo: ProjectConfigsRepository,
}

impl TaskBoardService {
    #[must_use]
    pub fn new(
        providers_repo: IntegrationProvidersRepository,
        project_configs_repo: ProjectConfigsRepository,
    ) -> Self {
        Self {
            providers_repo,
            project_configs_repo,
        }
    }

    pub async fn list_projects<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
    ) -> Result<Vec<TaskProviderProject>, TaskBoardError>
    where
        Q: Queryer<'c>,
    {
        let configs = self.project_configs_repo.list_all(db, team_id).await?;
        Ok(configs
            .into_iter()
            .filter_map(project_config_to_provider_project)
            .collect())
    }

    pub async fn get_board<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        provider_id: Uuid,
        external_project_id: &str,
    ) -> Result<TaskBoardResponse, TaskBoardError>
    where
        Q: Queryer<'c>,
    {
        let provider = self.load_provider(db, team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(&provider);
        let board = client.fetch_board(external_project_id).await?;

        Ok(TaskBoardResponse {
            provider_id: provider.id,
            provider_type: provider.provider_type,
            board,
        })
    }

    pub async fn create_task<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        provider_id: Uuid,
        external_project_id: &str,
        request: CreateTaskRequest,
    ) -> Result<CreateTaskResponse, TaskBoardError>
    where
        Q: Queryer<'c>,
    {
        let title = normalized_required(&request.title, TaskBoardError::EmptyTitle)?;
        let provider = self.load_provider(db, team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(&provider);
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

    pub async fn move_task<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        status: &str,
    ) -> Result<MoveTaskResponse, TaskBoardError>
    where
        Q: Queryer<'c>,
    {
        let next_status = normalized_required(status, TaskBoardError::EmptyStatus)?;
        let provider = self.load_provider(db, team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(&provider);

        client.update_task_status(task_id, &next_status).await?;

        Ok(MoveTaskResponse {
            task_id: task_id.to_owned(),
            status: next_status,
        })
    }

    async fn load_provider<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        provider_id: Uuid,
    ) -> Result<IntegrationProvider, TaskBoardError>
    where
        Q: Queryer<'c>,
    {
        self.providers_repo
            .find_by_id(db, provider_id, team_id)
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

#[must_use]
pub(crate) fn project_config_to_provider_project(
    config: ProjectConfig,
) -> Option<TaskProviderProject> {
    let provider_id = config.provider_id?;
    let fallback_name = config.external_project_id.clone();

    Some(TaskProviderProject {
        provider_id,
        provider_type: config.integration_type,
        workspace_id: config.external_workspace_id,
        external_project_id: config.external_project_id,
        name: if config.name.is_empty() {
            fallback_name.clone()
        } else {
            config.name
        },
        slug: fallback_name,
    })
}
#[must_use]
pub(crate) fn default_column_status(columns: &[IntegrationColumn]) -> String {
    columns
        .iter()
        .find(|column| column.is_final != Some(true))
        .or_else(|| columns.first())
        .map(|column| column.slug.to_owned())
        .unwrap_or_else(|| FALLBACK_STATUS.to_owned())
}

fn normalized_required(input: &str, err: TaskBoardError) -> Result<String, TaskBoardError> {
    let value = input.trim();
    if value.is_empty() {
        return Err(err);
    }

    Ok(value.to_owned())
}
