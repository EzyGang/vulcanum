use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::db::project_configs::ProjectConfigsRepository;
use crate::db::provider_configs::IntegrationProvidersRepository;
use crate::db::queryer::Queryer;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::project_configs::model::ProjectConfig;
use crate::models::provider_configs::model::IntegrationProvider;
use crate::models::providers::model::{
    CreateIntegrationTaskInput, IntegrationBoard, IntegrationColumn, UpdateIntegrationTaskInput,
};
use crate::models::task_board::errors::TaskBoardError;
use crate::models::task_board::model::{
    CreateTaskRequest, CreateTaskResponse, MoveTaskResponse, TaskBoardRelatedWorkRun,
    TaskBoardResponse, TaskBoardTaskRelatedRuns, TaskLabelDeleteResponse, TaskLabelResponse,
    TaskProviderProject, UpdateTaskRequest, UpdateTaskResponse,
};
use crate::models::work_runs::model::TaskBoardRelatedWorkRunRow;
use crate::services::providers::client::IntegrationClient;

const DEFAULT_PRIORITY: &str = "low";
const FALLBACK_STATUS: &str = "planned";
const RELATED_RUN_LIMIT: i64 = 3;

#[derive(Clone)]
pub struct TaskBoardService {
    providers_repo: IntegrationProvidersRepository,
    project_configs_repo: ProjectConfigsRepository,
    work_runs_repo: WorkRunsRepository,
}

impl TaskBoardService {
    #[must_use]
    pub fn new(
        providers_repo: IntegrationProvidersRepository,
        project_configs_repo: ProjectConfigsRepository,
        work_runs_repo: WorkRunsRepository,
    ) -> Self {
        Self {
            providers_repo,
            project_configs_repo,
            work_runs_repo,
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
        Q: Queryer<'c> + Copy,
    {
        let provider = self.load_provider(db, team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(&provider);
        let mut board = client.fetch_board(external_project_id).await?;
        let project = client.lookup_project(external_project_id).await?;
        if let Some(workspace_id) = project.workspace_id.as_deref() {
            board.labels = client.fetch_labels(workspace_id).await?;
        }
        let related_task_runs = self
            .related_task_runs(db, team_id, provider_id, external_project_id, &board)
            .await?;

        Ok(TaskBoardResponse {
            provider_id: provider.id,
            provider_type: provider.provider_type,
            board,
            related_task_runs,
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

    pub async fn update_task<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        request: UpdateTaskRequest,
    ) -> Result<UpdateTaskResponse, TaskBoardError>
    where
        Q: Queryer<'c>,
    {
        let title = normalized_required(&request.title, TaskBoardError::EmptyTitle)?;
        let provider = self.load_provider(db, team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(&provider);
        let task = client
            .update_task(UpdateIntegrationTaskInput {
                task_id: task_id.to_owned(),
                title,
                body: request.body,
            })
            .await?;

        Ok(UpdateTaskResponse { task })
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

    pub async fn add_task_label<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        label_id: &str,
    ) -> Result<TaskLabelResponse, TaskBoardError>
    where
        Q: Queryer<'c>,
    {
        let label_id = normalized_required(label_id, TaskBoardError::EmptyLabel)?;
        let provider = self.load_provider(db, team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(&provider);

        client.add_task_label(task_id, &label_id).await?;

        Ok(TaskLabelResponse {
            task_id: task_id.to_owned(),
            label_id,
        })
    }

    pub async fn remove_task_label<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        label_id: &str,
    ) -> Result<TaskLabelResponse, TaskBoardError>
    where
        Q: Queryer<'c>,
    {
        let label_id = normalized_required(label_id, TaskBoardError::EmptyLabel)?;
        let provider = self.load_provider(db, team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(&provider);

        client.remove_task_label(task_id, &label_id).await?;

        Ok(TaskLabelResponse {
            task_id: task_id.to_owned(),
            label_id,
        })
    }

    pub async fn delete_label<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        provider_id: Uuid,
        label_id: &str,
    ) -> Result<TaskLabelDeleteResponse, TaskBoardError>
    where
        Q: Queryer<'c>,
    {
        let label_id = normalized_required(label_id, TaskBoardError::EmptyLabel)?;
        let provider = self.load_provider(db, team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(&provider);
        client.delete_label(&label_id).await?;

        Ok(TaskLabelDeleteResponse { label_id })
    }

    async fn related_task_runs<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        provider_id: Uuid,
        external_project_id: &str,
        board: &IntegrationBoard,
    ) -> Result<Vec<TaskBoardTaskRelatedRuns>, TaskBoardError>
    where
        Q: Queryer<'c> + Copy,
    {
        let task_refs = collect_board_task_refs(board);
        if task_refs.is_empty() {
            return Ok(Vec::new());
        }

        let project_config = self
            .project_configs_repo
            .find_by_provider_project(db, team_id, provider_id, external_project_id)
            .await?;
        let project_config = match project_config {
            Some(config) => config,
            None => return Ok(Vec::new()),
        };
        let rows = self
            .work_runs_repo
            .list_latest_related_for_task_refs(
                db,
                team_id,
                project_config.id,
                &task_refs,
                RELATED_RUN_LIMIT,
            )
            .await?;

        Ok(group_related_runs(task_refs, rows))
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
        // Only fallback API callers omit status. Keep this uncached so provider column changes
        // do not require invalidation in Vulcanum.
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
pub(crate) fn collect_board_task_refs(board: &IntegrationBoard) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut task_refs = Vec::new();

    for column in &board.columns {
        for task in &column.tasks {
            let task_ref = task.id.clone();
            if seen.insert(task_ref.clone()) {
                task_refs.push(task_ref);
            }
        }
    }

    task_refs
}

#[must_use]
pub(crate) fn group_related_runs(
    task_refs: Vec<String>,
    rows: Vec<TaskBoardRelatedWorkRunRow>,
) -> Vec<TaskBoardTaskRelatedRuns> {
    let mut runs_by_ref: HashMap<String, Vec<TaskBoardRelatedWorkRun>> = HashMap::new();

    for row in rows {
        runs_by_ref
            .entry(row.external_task_ref.clone())
            .or_default()
            .push(TaskBoardRelatedWorkRun {
                id: row.id,
                status: row.status,
                work_type: row.work_type,
                tokens_used: row.tokens_used,
                input_tokens: row.input_tokens,
                output_tokens: row.output_tokens,
                cache_read_tokens: row.cache_read_tokens,
                cache_write_tokens: row.cache_write_tokens,
                model_used: row.model_used,
                created_at: row.created_at,
            });
    }

    task_refs
        .into_iter()
        .filter_map(|external_task_ref| {
            runs_by_ref
                .remove(&external_task_ref)
                .map(|runs| TaskBoardTaskRelatedRuns {
                    external_task_ref,
                    runs,
                })
        })
        .collect()
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
