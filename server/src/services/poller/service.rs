mod project;

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;

use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::provider_configs::repository::IntegrationProvidersRepository;
use crate::services::providers::client::TaskFetcher;
use crate::services::providers::errors::IntegrationError;
use crate::services::work_runs::repository::WorkRunsRepository;

#[derive(Debug)]
pub(super) enum PollError {
    Integration(IntegrationError),
}

impl fmt::Display for PollError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integration(e) => write!(f, "{}", e),
        }
    }
}

impl From<IntegrationError> for PollError {
    fn from(e: IntegrationError) -> Self {
        Self::Integration(e)
    }
}

pub struct PollerService {
    pub(super) project_configs: ProjectConfigsService,
    pub(super) work_runs_repo: WorkRunsRepository,
    pub(super) providers_repo: IntegrationProvidersRepository,
    pub(super) db: PgPool,
    poll_period: Duration,
    pub(super) task_fetcher: Option<Arc<dyn TaskFetcher>>,
}

impl PollerService {
    pub fn new(
        project_configs: ProjectConfigsService,
        work_runs_repo: WorkRunsRepository,
        providers_repo: IntegrationProvidersRepository,
        db: PgPool,
        poll_period_secs: u64,
    ) -> Self {
        Self {
            project_configs,
            work_runs_repo,
            providers_repo,
            db,
            poll_period: Duration::from_secs(poll_period_secs),
            task_fetcher: None,
        }
    }

    #[cfg(test)]
    pub fn with_fetcher(mut self, fetcher: Arc<dyn TaskFetcher>) -> Self {
        self.task_fetcher = Some(fetcher);
        self
    }

    pub async fn run(self) {
        let mut interval = tokio::time::interval(self.poll_period);

        loop {
            interval.tick().await;
            self.poll_once().await;
        }
    }

    pub(crate) async fn poll_once(&self) {
        tracing::debug!("Starting poll cycle");

        let configs = match self.project_configs.list_enabled().await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to list enabled project configs: {}", e);
                return;
            }
        };

        let project_count = configs.len();
        tracing::debug!(project_count, "loaded enabled project configs for polling");

        for config in &configs {
            match self.poll_project(config).await {
                Ok((tasks_found, inserted, skipped)) => {
                    tracing::debug!(
                        project_config_id = %config.id,
                        project_id = %config.external_project_id,
                        tasks_found,
                        tasks_inserted = inserted,
                        tasks_skipped = skipped,
                        "project poll complete",
                    );

                    if inserted > 0 {
                        tracing::info!(
                            project_count = project_count,
                            tasks_found = tasks_found,
                            tasks_inserted = inserted,
                            project_id = config.external_project_id.as_str(),
                            "Inserted {} new work_runs for project {}",
                            inserted,
                            config.external_project_id,
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(
                        project_id = config.external_project_id.as_str(),
                        "Integration poll failed for project {}: {}",
                        config.external_project_id,
                        e,
                    );
                }
            }
        }

        for config in &configs {
            if let Err(e) = self.reconcile_blocked_runs(config).await {
                tracing::warn!(
                    project_id = %config.external_project_id,
                    error = %e,
                    "blocked run reconciliation failed",
                );
            }
        }

        tracing::debug!(
            project_count = project_count,
            "Poll cycle complete, checked {} projects",
            project_count,
        );
    }
}

#[must_use]
pub(crate) fn repo_layout(repo_full_names: &[String]) -> String {
    project::repo_layout(repo_full_names)
}
