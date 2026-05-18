use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;

use crate::services::kaneo::client::TaskFetcher;
use crate::services::poller::notifier::WorkNotifier;
use crate::services::project_configs::model::ProjectConfig;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::work_runs::repository::work_runs::InsertWorkRunParams;
use crate::services::work_runs::repository::WorkRunsRepository;

pub struct PollerService {
    kaneo: Arc<dyn TaskFetcher>,
    project_configs_repo: ProjectConfigsRepository,
    work_runs_repo: WorkRunsRepository,
    db: PgPool,
    poll_period: Duration,
    notifier: WorkNotifier,
}

impl PollerService {
    pub fn new(
        kaneo: Arc<dyn TaskFetcher>,
        project_configs_repo: ProjectConfigsRepository,
        work_runs_repo: WorkRunsRepository,
        db: PgPool,
        poll_period_secs: u64,
        notifier: WorkNotifier,
    ) -> Self {
        Self {
            kaneo,
            project_configs_repo,
            work_runs_repo,
            db,
            poll_period: Duration::from_secs(poll_period_secs),
            notifier,
        }
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

        let configs = match self.project_configs_repo.list_enabled(&self.db).await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to list enabled project configs: {}", e);
                return;
            }
        };

        for config in &configs {
            match self.poll_project(config).await {
                Ok(inserted) => {
                    if inserted > 0 {
                        tracing::info!(
                            "Inserted {} new work_runs for project {}",
                            inserted,
                            config.kaneo_project_id,
                        );
                        self.notifier.notify_all().await;
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Kaneo poll failed for project {}: {}",
                        config.kaneo_project_id,
                        e,
                    );
                }
            }
        }

        tracing::debug!("Poll cycle complete, checked {} projects", configs.len());
    }

    async fn poll_project(&self, config: &ProjectConfig) -> Result<usize, String> {
        let tasks = self
            .kaneo
            .fetch_tasks_in_column(&config.kaneo_project_id, &config.pickup_column)
            .await
            .map_err(|e| format!("{}", e))?;

        let mut inserted = 0;
        for task in &tasks {
            let params = InsertWorkRunParams {
                external_task_ref: task.id.clone(),
                project_config_id: config.id,
                prompt_text: config.prompt_template.clone(),
                status: "pending".to_owned(),
            };

            match self
                .work_runs_repo
                .insert_work_run_if_not_active(&self.db, params)
                .await
            {
                Ok(true) => inserted += 1,
                Ok(false) => (),
                Err(e) => {
                    tracing::error!("Failed to insert work_run for task {}: {}", task.id, e);
                }
            }
        }

        Ok(inserted)
    }
}
