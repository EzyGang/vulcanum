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
pub(crate) mod reconcile_pr_completion;
pub(crate) mod record_review;
pub mod refresh_github_token;
pub(crate) mod request_github_review;
pub(crate) mod review_ticket;
pub mod spawn_review;
pub mod submit_result;
pub(crate) mod sync_task_tracker;
pub(crate) mod worker_accounting;

use std::sync::Arc;

use sqlx::PgPool;

use crate::db::project_usage::ProjectUsageRepository;
use crate::db::provider_configs::IntegrationProvidersRepository;
use crate::db::task_augmentations::TaskAugmentationsRepository;
use crate::db::work_runs::WorkRunsRepository;
use crate::db::workers::WorkersRepository;
use crate::services::dispatcher::cancel_store::CancelStore;
use crate::services::dispatcher::dispatch_store::DispatchStore;
use crate::services::github_app::service::pull_requests::PullRequestStateReader;
use crate::services::github_app::service::GithubAppManager;
use crate::services::model_providers::service::ModelProvidersService;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::providers::client::TaskFetcher;
use crate::services::work_runs::service::review_ticket::{
    IntegrationReviewTicketCreator, ReviewTicketCreator,
};

pub struct WorkRunsService {
    pub work_runs_repo: WorkRunsRepository,
    pub task_augmentations_repo: TaskAugmentationsRepository,
    pub project_usage_repo: ProjectUsageRepository,
    pub workers_repo: WorkersRepository,
    pub project_configs: ProjectConfigsService,
    pub github: GithubAppManager,
    pub db: PgPool,
    pub(crate) pr_state_reader: Arc<dyn PullRequestStateReader>,
    review_ticket_creator: Arc<dyn ReviewTicketCreator>,
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
            project_usage_repo: self.project_usage_repo.clone(),
            workers_repo: self.workers_repo.clone(),
            project_configs: self.project_configs.clone(),
            github: self.github.clone(),
            pr_state_reader: self.pr_state_reader.clone(),
            review_ticket_creator: self.review_ticket_creator.clone(),
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
        project_usage_repo: ProjectUsageRepository,
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
        let pr_state_reader: Arc<dyn PullRequestStateReader> = Arc::new(github.clone());
        let review_ticket_creator: Arc<dyn ReviewTicketCreator> =
            Arc::new(IntegrationReviewTicketCreator);

        Self {
            work_runs_repo,
            task_augmentations_repo,
            project_usage_repo,
            workers_repo,
            project_configs,
            github,
            db,
            pr_state_reader,
            review_ticket_creator,
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

    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_pr_state_reader(
        mut self,
        pr_state_reader: Arc<dyn PullRequestStateReader>,
    ) -> Self {
        self.pr_state_reader = pr_state_reader;
        self
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_review_ticket_creator(
        mut self,
        review_ticket_creator: Arc<dyn ReviewTicketCreator>,
    ) -> Self {
        self.review_ticket_creator = review_ticket_creator;
        self
    }
}
