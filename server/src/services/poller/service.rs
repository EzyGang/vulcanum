use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;

use crate::services::integrations::client::TaskFetcher;
use crate::services::integrations::errors::IntegrationError;
use crate::services::project_configs::model::ProjectConfig;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::work_runs::model::WorkRunStatus;
use crate::services::work_runs::repository::work_runs::InsertWorkRunParams;
use crate::services::work_runs::repository::WorkRunsRepository;

#[derive(Debug)]
enum PollError {
    Integration(IntegrationError),
}

impl std::fmt::Display for PollError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    integration: Arc<dyn TaskFetcher>,
    project_configs_repo: ProjectConfigsRepository,
    work_runs_repo: WorkRunsRepository,
    db: PgPool,
    poll_period: Duration,
}

impl PollerService {
    pub fn new(
        integration: Arc<dyn TaskFetcher>,
        project_configs_repo: ProjectConfigsRepository,
        work_runs_repo: WorkRunsRepository,
        db: PgPool,
        poll_period_secs: u64,
    ) -> Self {
        Self {
            integration,
            project_configs_repo,
            work_runs_repo,
            db,
            poll_period: Duration::from_secs(poll_period_secs),
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

        let project_count = configs.len();

        for config in &configs {
            match self.poll_project(config).await {
                Ok((tasks_found, inserted)) => {
                    if inserted > 0 {
                        tracing::info!(
                            project_count = project_count,
                            tasks_found = tasks_found,
                            tasks_inserted = inserted,
                            project_id = config.kaneo_project_id.as_str(),
                            "Inserted {} new work_runs for project {}",
                            inserted,
                            config.kaneo_project_id,
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(
                        project_id = config.kaneo_project_id.as_str(),
                        "Integration poll failed for project {}: {}",
                        config.kaneo_project_id,
                        e,
                    );
                }
            }
        }

        tracing::debug!(
            project_count = project_count,
            "Poll cycle complete, checked {} projects",
            project_count,
        );
    }

    async fn poll_project(&self, config: &ProjectConfig) -> Result<(usize, usize), PollError> {
        let tasks = self
            .integration
            .fetch_tasks_in_column(&config.kaneo_project_id, &config.pickup_column)
            .await?;

        let tasks_found = tasks.len();
        let mut inserted = 0;

        for task in &tasks {
            let prompt_text = crate::services::poller::template::render_template(
                &config.prompt_template,
                &crate::services::poller::template::TemplateVars {
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

        Ok((tasks_found, inserted))
    }
}
