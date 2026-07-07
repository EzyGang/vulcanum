pub mod acknowledge;
pub mod cancel;
pub mod clear_cancel_flag;
pub mod delete;
pub mod fail;
pub mod get_job;
#[cfg(test)]
mod get_job_tests;
pub(crate) mod lifecycle_labels;
pub mod list;
pub mod poll;
pub(crate) mod record_review;
pub mod refresh_github_token;
pub mod spawn_review;
pub mod submit_result;
pub(crate) mod sync_task_tracker;

use std::sync::Arc;

use sqlx::PgPool;

use crate::db::provider_configs::IntegrationProvidersRepository;
use crate::db::task_augmentations::TaskAugmentationsRepository;
use crate::db::work_runs::WorkRunsRepository;
use crate::db::workers::WorkersRepository;
use crate::services::dispatcher::cancel_store::CancelStore;
use crate::services::dispatcher::dispatch_store::DispatchStore;
use crate::services::github_app::service::GithubAppManager;
use crate::services::model_providers::service::ModelProvidersService;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::providers::client::TaskFetcher;

pub struct WorkRunsService {
    pub work_runs_repo: WorkRunsRepository,
    pub task_augmentations_repo: TaskAugmentationsRepository,
    pub workers_repo: WorkersRepository,
    pub project_configs: ProjectConfigsService,
    pub github: GithubAppManager,
    pub db: PgPool,
    dispatch_store: Arc<dyn DispatchStore>,
    cancel_store: Arc<dyn CancelStore>,
    pub providers_repo: IntegrationProvidersRepository,
    pub model_providers: ModelProvidersService,
    pub unhealthy_threshold: i32,
    pub task_fetcher: Option<Arc<dyn TaskFetcher>>,
}

impl Clone for WorkRunsService {
    fn clone(&self) -> Self {
        Self {
            work_runs_repo: self.work_runs_repo.clone(),
            task_augmentations_repo: self.task_augmentations_repo.clone(),
            workers_repo: self.workers_repo.clone(),
            project_configs: self.project_configs.clone(),
            github: self.github.clone(),
            db: self.db.clone(),
            dispatch_store: self.dispatch_store.clone(),
            cancel_store: self.cancel_store.clone(),
            providers_repo: self.providers_repo.clone(),
            model_providers: self.model_providers.clone(),
            unhealthy_threshold: self.unhealthy_threshold,
            task_fetcher: self.task_fetcher.clone(),
        }
    }
}

impl WorkRunsService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        work_runs_repo: WorkRunsRepository,
        task_augmentations_repo: TaskAugmentationsRepository,
        workers_repo: WorkersRepository,
        project_configs: ProjectConfigsService,
        github: GithubAppManager,
        db: PgPool,
        dispatch_store: Arc<dyn DispatchStore>,
        providers_repo: IntegrationProvidersRepository,
        model_providers: ModelProvidersService,
        cancel_store: Arc<dyn CancelStore>,
        unhealthy_threshold: i32,
    ) -> Self {
        Self {
            work_runs_repo,
            task_augmentations_repo,
            workers_repo,
            project_configs,
            github,
            db,
            dispatch_store,
            cancel_store,
            providers_repo,
            model_providers,
            unhealthy_threshold,
            task_fetcher: None,
        }
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn dispatch_store(&self) -> Arc<dyn DispatchStore> {
        self.dispatch_store.clone()
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn cancel_store(&self) -> Arc<dyn CancelStore> {
        self.cancel_store.clone()
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_task_fetcher(mut self, task_fetcher: Arc<dyn TaskFetcher>) -> Self {
        self.task_fetcher = Some(task_fetcher);
        self
    }
}
