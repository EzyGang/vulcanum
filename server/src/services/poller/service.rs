use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;

use crate::services::project_configs::model::{EffectiveProjectSettings, ProjectConfig};
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::provider_configs::repository::IntegrationProvidersRepository;
use crate::services::providers::client::{IntegrationClient, TaskFetcher};
use crate::services::providers::errors::IntegrationError;
use crate::services::providers::model::{IntegrationTask, IntegrationType};
use crate::services::teams::repository::TeamsRepository;
use crate::services::work_runs::model::WorkRunStatus;
use crate::services::work_runs::repository::queries::InsertWorkRunParams;
use crate::services::work_runs::repository::WorkRunsRepository;

use super::prompts::{ENVIRONMENT_INSTRUCTION, GITHUB_INSTRUCTION};

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
    project_configs_repo: ProjectConfigsRepository,
    work_runs_repo: WorkRunsRepository,
    providers_repo: IntegrationProvidersRepository,
    db: PgPool,
    poll_period: Duration,
    task_fetcher: Option<Arc<dyn TaskFetcher>>,
}

impl PollerService {
    pub fn new(
        project_configs_repo: ProjectConfigsRepository,
        work_runs_repo: WorkRunsRepository,
        providers_repo: IntegrationProvidersRepository,
        db: PgPool,
        poll_period_secs: u64,
    ) -> Self {
        Self {
            project_configs_repo,
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

    async fn poll_project(&self, config: &ProjectConfig) -> Result<(usize, usize), PollError> {
        let settings = match self.effective_settings(config).await {
            Ok(settings) => settings,
            Err(e) => {
                tracing::warn!(
                    project_config_id = %config.id,
                    error = %e,
                    "skipping poll — failed to resolve effective settings"
                );
                return Ok((0, 0));
            }
        };
        let provider_id = match config.provider_id {
            Some(pid) => pid,
            None => {
                tracing::warn!(
                    project_id = %config.external_project_id,
                    "skipping poll — no provider configured for project"
                );
                return Ok((0, 0));
            }
        };

        let provider = match self
            .providers_repo
            .find_by_id(&self.db, provider_id, config.team_id)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(
                    provider_id = %provider_id,
                    project_id = %config.external_project_id,
                    error = %e,
                    "skipping poll — provider not found"
                );
                return Ok((0, 0));
            }
        };

        let fetcher: Arc<dyn TaskFetcher> = match &self.task_fetcher {
            Some(f) => Arc::clone(f),
            None => {
                let client = match provider.provider_type {
                    IntegrationType::Kaneo => {
                        IntegrationClient::new_kaneo(provider.instance_url, provider.api_key)
                    }
                };
                Arc::new(client)
            }
        };

        let tasks = fetcher
            .fetch_tasks_in_column(&config.external_project_id, &config.pickup_column)
            .await?;

        let tasks_found = tasks.len();
        let mut inserted = 0;

        for task in &tasks {
            let repo_urls = config.repo_urls.join("\n");
            let repo_names = config.repo_full_names.join("\n");
            let repo_layout = repo_layout(&config.repo_full_names);
            let mut prompt_text = crate::services::poller::template::render_template(
                &settings.prompt_template,
                &crate::services::poller::template::TemplateVars {
                    task_title: &task.title,
                    task_body: task.description.as_deref().unwrap_or(""),
                    repo_url: &config.repo_url,
                    repo_urls: &repo_urls,
                    repo_names: &repo_names,
                    repo_layout: &repo_layout,
                },
            );

            prompt_text.push_str(ENVIRONMENT_INSTRUCTION);

            if !config.repo_full_names.is_empty() {
                prompt_text.push_str(GITHUB_INSTRUCTION);
            }
            let task_slug = build_task_slug(task);
            let params = InsertWorkRunParams {
                team_id: config.team_id,
                external_task_ref: task.id.clone(),
                project_config_id: config.id,
                prompt_text,
                repo_url: config.repo_url.clone(),
                repo_full_names: config.repo_full_names.clone(),
                agents_md: settings.agents_md.clone(),
                status: WorkRunStatus::Pending,
                task_title: Some(task.title.clone()),
                task_slug,
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

    async fn effective_settings(
        &self,
        config: &ProjectConfig,
    ) -> Result<EffectiveProjectSettings, crate::services::teams::errors::TeamsError> {
        let team = TeamsRepository::new()
            .get_by_id(&self.db, config.team_id)
            .await?;
        Ok(EffectiveProjectSettings {
            prompt_template: config
                .prompt_template
                .clone()
                .unwrap_or(team.prompt_template),
            agents_md: config.agents_md.clone().unwrap_or(team.agents_md),
            primary_model_provider_key: config
                .primary_model_provider_key
                .clone()
                .or(team.primary_model_provider_key),
            primary_model_id: config.primary_model_id.clone().or(team.primary_model_id),
            small_model_provider_key: config
                .small_model_provider_key
                .clone()
                .or(team.small_model_provider_key),
            small_model_id: config.small_model_id.clone().or(team.small_model_id),
        })
    }

    async fn reconcile_blocked_runs(&self, config: &ProjectConfig) -> Result<(), PollError> {
        let provider_id = match config.provider_id {
            Some(pid) => pid,
            None => return Ok(()),
        };

        let provider = match self
            .providers_repo
            .find_by_id(&self.db, provider_id, config.team_id)
            .await
        {
            Ok(p) => p,
            Err(_) => return Ok(()),
        };

        let client: Arc<dyn TaskFetcher> = Arc::new(match provider.provider_type {
            IntegrationType::Kaneo => {
                IntegrationClient::new_kaneo(provider.instance_url, provider.api_key)
            }
        });

        let blocked_runs = self
            .work_runs_repo
            .find_blocked_by_project(&self.db, config.id)
            .await
            .unwrap_or_default();

        for run in &blocked_runs {
            let tasks = client
                .fetch_tasks_in_column(&config.external_project_id, &config.pickup_column)
                .await?;

            if tasks.iter().any(|t| t.id == run.external_task_ref) {
                self.work_runs_repo
                    .reset_blocked_to_pending(&self.db, run.id)
                    .await
                    .unwrap_or_else(|e| {
                        tracing::warn!(
                            work_run_id = %run.id,
                            error = %e,
                            "failed to unblock work run",
                        );
                    });
            }
        }

        Ok(())
    }
}

fn repo_layout(repo_full_names: &[String]) -> String {
    repo_full_names
        .iter()
        .map(|name| format!("{name}: ./{name}"))
        .collect::<Vec<String>>()
        .join("\n")
}

fn build_task_slug(task: &IntegrationTask) -> Option<String> {
    let project_slug = task.project_slug.as_deref()?;
    let number = match task.number {
        Some(n) => n.to_string(),
        None => {
            let id_prefix = &task.id[..task.id.len().min(8)];
            id_prefix.to_owned()
        }
    };
    Some(format!("{project_slug}-{number}"))
}
