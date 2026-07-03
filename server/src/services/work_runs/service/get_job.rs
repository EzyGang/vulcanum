use uuid::Uuid;
use vulcanum_shared::api_types::JobResponse;

use crate::models::project_configs::model::JobConfigFields;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRun, WorkRunType};
use crate::services::model_providers::renderer::ModelSelection;
use crate::services::poller::service::{render_work_run_prompt, repo_layout, RenderPromptInput};
use crate::services::providers::client::IntegrationClient;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn get_job(&self, id: Uuid, worker_id: Uuid) -> Result<JobResponse, WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;
        if run.worker_id != Some(worker_id) {
            return Err(WorkRunsError::NotOwned);
        }

        let config = self.project_configs.find_by_id(run.project_config_id).await;

        let cfg = match &config {
            Ok(c) => {
                let settings = self.project_configs.effective_settings(c).await?;
                c.job_fields(settings)
            }
            Err(_) => {
                tracing::warn!(
                    project_config_id = %run.project_config_id,
                    work_run_id = %id,
                    "project config not found for work run"
                );
                JobConfigFields::empty_for_team(run.team_id)
            }
        };
        let repos = self.github_repos_for_work_run(&run).await?;
        let pr_urls = self.work_runs_repo.list_pr_urls(&self.db, id).await?;
        let task = self.fetch_task_prompt_data(&run, cfg.provider_id).await;
        let repo_full_names = repos
            .iter()
            .map(|repo| repo.full_name.clone())
            .collect::<Vec<String>>();
        let repo_names = repo_full_names.join("\n");
        let repo_urls = repos
            .iter()
            .map(|repo| repo.url.clone())
            .collect::<Vec<String>>()
            .join("\n");
        let rendered_repo_layout = repo_layout(&repo_full_names);
        let repo_url = repos
            .first()
            .map(|repo| repo.url.as_str())
            .unwrap_or_default();
        let prompt_template = match run.work_type {
            WorkRunType::Implementation => cfg.prompt_template.as_str(),
            WorkRunType::PullRequestReview => cfg.review_prompt_template.as_str(),
        };
        let prompt_text = render_work_run_prompt(RenderPromptInput {
            prompt_template,
            task_title: &task.title,
            task_body: &task.body,
            repo_url,
            repo_urls: &repo_urls,
            repo_names: &repo_names,
            repo_layout: &rendered_repo_layout,
            review_target_pr_url: run.review_target_pr_url.as_deref().unwrap_or_default(),
            has_github_repos: !repos.is_empty(),
        });

        let (provider_instance_url, provider_api_key) = match cfg.provider_id {
            Some(pid) => match self
                .providers_repo
                .find_by_id(&self.db, pid, cfg.team_id)
                .await
            {
                Ok(provider) => (provider.instance_url, provider.api_key),
                Err(_) => (String::new(), String::new()),
            },
            None => (String::new(), String::new()),
        };

        let github_token = self
            .mint_github_token_for_repos(id, cfg.team_id, &repos)
            .await?;

        let team = self.project_configs.teams.get_team(cfg.team_id).await?;
        let rendered = self
            .model_providers
            .render_agent_config_for_team(
                cfg.team_id,
                cfg.agent_backend,
                ModelSelection {
                    primary_provider_key: team.primary_model_provider_key.as_deref(),
                    primary_model_id: team.primary_model_id.as_deref(),
                    small_provider_key: team.small_model_provider_key.as_deref(),
                    small_model_id: team.small_model_id.as_deref(),
                },
            )
            .await?;

        Ok(JobResponse {
            work_type: shared_work_type(run.work_type),
            prompt_text,
            repos,
            agents_md: cfg.agents_md,
            agent_backend: cfg.agent_backend,
            agent_config: rendered.agent_config,
            model_provider_env: rendered.env,
            external_task_ref: run.external_task_ref,
            provider_instance_url,
            provider_api_key,
            external_project_id: cfg.external_project_id,
            external_workspace_id: cfg.external_workspace_id,
            max_turns: match run.work_type {
                WorkRunType::Implementation => cfg.max_turns,
                WorkRunType::PullRequestReview => cfg.review_max_turns,
            },
            github_token: github_token.github_token,
            github_token_expires_at: github_token.github_token_expires_at,
            pr_urls,
            review_target_pr_url: run.review_target_pr_url,
            review_target_repo_full_name: run.review_target_repo_full_name,
        })
    }

    pub(crate) async fn fetch_task_prompt_data(
        &self,
        run: &WorkRun,
        provider_id: Option<Uuid>,
    ) -> TaskPromptData {
        let provider_id = match provider_id {
            Some(provider_id) => provider_id,
            None => return TaskPromptData::default(),
        };
        let provider = match self
            .providers_repo
            .find_by_id(&self.db, provider_id, run.team_id)
            .await
        {
            Ok(provider) => provider,
            Err(error) => {
                tracing::warn!(
                    work_run_id = %run.id,
                    provider_id = %provider_id,
                    error = %error,
                    "failed to load provider for task reconstruction",
                );
                return TaskPromptData::default();
            }
        };
        let client = IntegrationClient::from_provider(&provider);

        match client.fetch_task(&run.external_task_ref).await {
            Ok(task) => TaskPromptData {
                title: task.title,
                body: task.description.unwrap_or_default(),
            },
            Err(error) => {
                tracing::warn!(
                    work_run_id = %run.id,
                    task_ref = %run.external_task_ref,
                    error = %error,
                    "failed to fetch task for prompt reconstruction",
                );
                TaskPromptData::default()
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct TaskPromptData {
    pub title: String,
    pub body: String,
}

#[must_use]
fn shared_work_type(work_type: WorkRunType) -> vulcanum_shared::api_types::WorkRunType {
    match work_type {
        WorkRunType::Implementation => vulcanum_shared::api_types::WorkRunType::Implementation,
        WorkRunType::PullRequestReview => {
            vulcanum_shared::api_types::WorkRunType::PullRequestReview
        }
    }
}
