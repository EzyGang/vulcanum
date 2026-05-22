use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;

use crate::services::kaneo::client::TaskFetcher;
use crate::services::kaneo::errors::KaneoError;
use crate::services::poller::notifier::WorkNotifier;
use crate::services::poller::template::{self, TemplateVars};
use crate::services::project_configs::model::ProjectConfig;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::work_runs::model::WorkRunStatus;
use crate::services::work_runs::repository::work_runs::InsertWorkRunParams;
use crate::services::work_runs::repository::WorkRunsRepository;

#[derive(Debug)]
enum PollError {
    Kaneo(KaneoError),
}

impl std::fmt::Display for PollError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kaneo(e) => write!(f, "{}", e),
        }
    }
}

impl From<KaneoError> for PollError {
    fn from(e: KaneoError) -> Self {
        Self::Kaneo(e)
    }
}

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

    async fn poll_project(&self, config: &ProjectConfig) -> Result<usize, PollError> {
        let tasks = self
            .kaneo
            .fetch_tasks_in_column(&config.kaneo_project_id, &config.pickup_column)
            .await?;

        let mut inserted = 0;
        for task in &tasks {
            let prompt_text = template::render_template(
                &config.prompt_template,
                &TemplateVars {
                    task_title: &task.title,
                    task_body: task.description.as_deref().unwrap_or(""),
                    repo_url: &config.repo_url,
                },
            );
            let params = InsertWorkRunParams {
                external_task_ref: task.id.clone(),
                project_config_id: config.id,
                prompt_text,
                repo_url: config.repo_url.clone(),
                agents_md: config.agents_md.clone(),
                status: WorkRunStatus::Pending,
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
