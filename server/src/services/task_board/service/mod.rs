mod board;
mod labels;
mod lookups;
mod tasks;

mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use crate::db::project_configs::ProjectConfigsRepository;
use crate::db::project_usage::ProjectUsageRepository;
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
#[cfg(test)]
pub(crate) use crate::services::task_board::service::tasks::task_update_input;

const DEFAULT_PRIORITY: &str = "low";

#[derive(Clone)]
pub struct TaskBoardService {
    db: PgPool,
    providers_repo: IntegrationProvidersRepository,
    project_configs_repo: ProjectConfigsRepository,
    task_augmentations_repo: TaskAugmentationsRepository,
    project_usage_repo: ProjectUsageRepository,
}

impl TaskBoardService {
    #[must_use]
    pub fn new(
        db: PgPool,
        providers_repo: IntegrationProvidersRepository,
        project_configs_repo: ProjectConfigsRepository,
        task_augmentations_repo: TaskAugmentationsRepository,
        project_usage_repo: ProjectUsageRepository,
    ) -> Self {
        Self {
            db,
            providers_repo,
            project_configs_repo,
            task_augmentations_repo,
            project_usage_repo,
        }
    }
}
