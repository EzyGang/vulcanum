mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use crate::db::project_configs::ProjectConfigsRepository;
use crate::db::provider_configs::IntegrationProvidersRepository;
use crate::db::task_augmentations::TaskAugmentationsRepository;
use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::project_configs::model::ProjectConfig;
use crate::models::provider_configs::model::IntegrationProvider;
use crate::models::providers::model::{
    CreateIntegrationTaskInput, IntegrationBoard, IntegrationTask, UpdateIntegrationTaskInput,
};
use crate::models::task_board::errors::TaskBoardError;
use crate::models::task_board::model::{
    CreateTaskRequest, CreateTaskResponse, MoveTaskResponse, TaskBoardResponse,
    TaskBoardTaskAugmentation, TaskLabelDeleteResponse, TaskLabelResponse, TaskProviderProject,
    UpdateTaskRequest, UpdateTaskResponse,
};
use crate::services::providers::client::IntegrationClient;
#[cfg(test)]
pub(crate) use crate::services::task_board::service::helpers::default_column_status;
pub(crate) use crate::services::task_board::service::helpers::{
    collect_board_task_refs, project_config_to_provider_project,
};
use crate::services::task_board::service::helpers::{default_task_status, normalized_required};

const DEFAULT_PRIORITY: &str = "low";

#[derive(Clone)]
pub struct TaskBoardService {
    db: PgPool,
    providers_repo: IntegrationProvidersRepository,
    project_configs_repo: ProjectConfigsRepository,
    task_augmentations_repo: TaskAugmentationsRepository,
}

impl TaskBoardService {
    #[must_use]
    pub fn new(
        db: PgPool,
        providers_repo: IntegrationProvidersRepository,
        project_configs_repo: ProjectConfigsRepository,
        task_augmentations_repo: TaskAugmentationsRepository,
    ) -> Self {
        Self {
            db,
            providers_repo,
            project_configs_repo,
            task_augmentations_repo,
        }
    }

    pub async fn list_projects(
        &self,
        team_id: Uuid,
    ) -> Result<Vec<TaskProviderProject>, TaskBoardError> {
        let configs = self
            .project_configs_repo
            .list_all(&self.db, team_id)
            .await?;
        Ok(configs
            .into_iter()
            .filter_map(project_config_to_provider_project)
            .collect())
    }

    pub async fn get_board(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        external_project_id: &str,
    ) -> Result<TaskBoardResponse, TaskBoardError> {
        let (project_config, provider) = self
            .load_project_provider(team_id, provider_id, external_project_id)
            .await?;
        let client = IntegrationClient::from_provider(&provider);
        let mut board = client.fetch_board(external_project_id).await?;
        let project = client.lookup_project(external_project_id).await?;
        if let Some(workspace_id) = project.workspace_id.as_deref() {
            board.labels = client.fetch_labels(workspace_id).await?;
        }
        let task_augmentations = self
            .task_augmentations(team_id, project_config.id, &board)
            .await?;

        Ok(TaskBoardResponse {
            provider_id: provider.id,
            provider_type: provider.provider_type,
            board,
            task_augmentations,
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
        let (_, provider) = self
            .load_project_provider(team_id, provider_id, external_project_id)
            .await?;
        let client = IntegrationClient::from_provider(&provider);
        let status = match request.status.as_deref().map(str::trim) {
            Some(value) if !value.is_empty() => value.to_owned(),
            _ => default_task_status(&client, external_project_id).await?,
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

    pub async fn update_task(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        request: UpdateTaskRequest,
    ) -> Result<UpdateTaskResponse, TaskBoardError> {
        let title = normalized_required(&request.title, TaskBoardError::EmptyTitle)?;
        let (client, _) = self
            .load_task_provider(team_id, provider_id, task_id)
            .await?;
        let task = client
            .update_task(UpdateIntegrationTaskInput {
                task_id: task_id.to_owned(),
                title,
                body: request.body,
            })
            .await?;

        Ok(UpdateTaskResponse { task })
    }

    pub async fn move_task(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        status: &str,
    ) -> Result<MoveTaskResponse, TaskBoardError> {
        let next_status = normalized_required(status, TaskBoardError::EmptyStatus)?;
        let (client, _) = self
            .load_task_provider(team_id, provider_id, task_id)
            .await?;

        client.update_task_status(task_id, &next_status).await?;

        Ok(MoveTaskResponse {
            task_id: task_id.to_owned(),
            status: next_status,
        })
    }

    pub async fn add_task_label(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        label_id: &str,
    ) -> Result<TaskLabelResponse, TaskBoardError> {
        let label_id = normalized_required(label_id, TaskBoardError::EmptyLabel)?;
        let (client, _) = self
            .load_task_provider(team_id, provider_id, task_id)
            .await?;

        client.add_task_label(task_id, &label_id).await?;

        Ok(TaskLabelResponse {
            task_id: task_id.to_owned(),
            label_id,
        })
    }

    pub async fn remove_task_label(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        label_id: &str,
    ) -> Result<TaskLabelResponse, TaskBoardError> {
        let label_id = normalized_required(label_id, TaskBoardError::EmptyLabel)?;
        let (client, _) = self
            .load_task_provider(team_id, provider_id, task_id)
            .await?;

        client.remove_task_label(task_id, &label_id).await?;

        Ok(TaskLabelResponse {
            task_id: task_id.to_owned(),
            label_id,
        })
    }

    pub async fn delete_label(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        label_id: &str,
    ) -> Result<TaskLabelDeleteResponse, TaskBoardError> {
        let label_id = normalized_required(label_id, TaskBoardError::EmptyLabel)?;
        let provider = self.load_configured_provider(team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(&provider);
        client.delete_label(&label_id).await?;

        Ok(TaskLabelDeleteResponse { label_id })
    }

    async fn task_augmentations(
        &self,
        team_id: Uuid,
        project_config_id: Uuid,
        board: &IntegrationBoard,
    ) -> Result<Vec<TaskBoardTaskAugmentation>, TaskBoardError> {
        let task_refs = collect_board_task_refs(board);
        if task_refs.is_empty() {
            return Ok(Vec::new());
        }

        self.task_augmentations_repo
            .list_for_task_refs(&self.db, team_id, project_config_id, &task_refs)
            .await
            .map_err(TaskBoardError::from)
    }

    async fn load_project_provider(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        external_project_id: &str,
    ) -> Result<(ProjectConfig, IntegrationProvider), TaskBoardError> {
        let project_config = self
            .project_configs_repo
            .find_by_provider_project(&self.db, team_id, provider_id, external_project_id)
            .await?
            .ok_or(ProjectConfigsError::NotFound)?;
        let provider = self.load_provider(team_id, provider_id).await?;

        Ok((project_config, provider))
    }

    async fn load_task_provider(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
    ) -> Result<(IntegrationClient, IntegrationTask), TaskBoardError> {
        let provider = self.load_provider(team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(&provider);
        let task = client.fetch_task(task_id).await?;
        self.project_configs_repo
            .find_by_provider_project(&self.db, team_id, provider_id, &task.project_id)
            .await?
            .ok_or(ProjectConfigsError::NotFound)?;

        Ok((client, task))
    }

    async fn load_configured_provider(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
    ) -> Result<IntegrationProvider, TaskBoardError> {
        let provider = self.load_provider(team_id, provider_id).await?;
        let configs = self
            .project_configs_repo
            .list_all(&self.db, team_id)
            .await?;
        if !configs
            .iter()
            .any(|config| config.provider_id == Some(provider_id))
        {
            return Err(ProjectConfigsError::NotFound.into());
        }

        Ok(provider)
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
}
