use uuid::Uuid;
use vulcanum_shared::api::wire::{JobRepo, JobResponse};

use crate::models::project_configs::model::JobConfigFields;
use crate::models::providers::model::IntegrationTask;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRun, WorkRunType};
use crate::services::model_providers::renderer::ModelSelection;
use crate::services::poller::prompts::{
    ENVIRONMENT_INSTRUCTION, GITHUB_INSTRUCTION, REVIEW_GITHUB_INSTRUCTION,
};
use crate::services::poller::service::repo_layout;
use crate::services::poller::template::{render_template, TemplateVars};
use crate::services::providers::client::IntegrationClient;
use crate::services::work_runs::service::WorkRunsService;
use crate::util::github::github_repo_url;

impl WorkRunsService {
    pub async fn get_job(&self, id: Uuid, worker_id: Uuid) -> Result<JobResponse, WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;
        if run.worker_id != Some(worker_id) {
            return Err(WorkRunsError::NotOwned);
        }

        let cfg = self.job_config_fields_for_run(&run).await?;
        let task = self.fetch_task_for_run(&run, &cfg).await?;
        let repos = self.github_repos_for_work_run(&run).await?;
        let pr_urls = self.work_runs_repo.list_pr_urls(&self.db, id).await?;

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
            prompt_text: render_prompt_text(&run, &cfg, &task, &repos),
            repos,
            agents_md: cfg.agents_md.clone(),
            agent_backend: cfg.agent_backend,
            agent_config: rendered.agent_config,
            model_provider_env: rendered.env,
            external_task_ref: run.external_task_ref.clone(),
            provider_instance_url,
            provider_api_key,
            external_project_id: cfg.external_project_id.clone(),
            external_workspace_id: cfg.external_workspace_id.clone(),
            max_turns: match run.work_type {
                WorkRunType::Implementation => cfg.max_turns,
                WorkRunType::PullRequestReview => cfg.review_max_turns,
            },
            github_token: github_token.github_token,
            github_token_expires_at: github_token.github_token_expires_at,
            pr_urls,
            review_target_pr_url: run.review_target_pr_url.clone(),
            review_target_repo_full_name: run.review_target_repo_full_name.clone(),
        })
    }

    pub(crate) async fn job_config_fields_for_run(
        &self,
        run: &WorkRun,
    ) -> Result<JobConfigFields, WorkRunsError> {
        match self.project_configs.find_by_id(run.project_config_id).await {
            Ok(config) => {
                let settings = self.project_configs.effective_settings(&config).await?;
                Ok(config.job_fields(settings))
            }
            Err(_) => {
                tracing::warn!(
                    project_config_id = %run.project_config_id,
                    work_run_id = %run.id,
                    "project config not found for work run"
                );
                Ok(JobConfigFields::empty_for_team(run.team_id))
            }
        }
    }

    pub(crate) async fn fetch_task_for_run(
        &self,
        run: &WorkRun,
        cfg: &JobConfigFields,
    ) -> Result<IntegrationTask, WorkRunsError> {
        if let Some(fetcher) = &self.task_fetcher {
            return Ok(fetcher.fetch_task(&run.external_task_ref).await?);
        }

        let provider_id = match cfg.provider_id {
            Some(provider_id) => provider_id,
            None => {
                tracing::warn!(
                    work_run_id = %run.id,
                    task_ref = %run.external_task_ref,
                    "reconstructing work run prompt without provider task data"
                );
                return Ok(empty_task(run, cfg));
            }
        };

        let provider = self
            .providers_repo
            .find_by_id(&self.db, provider_id, cfg.team_id)
            .await
            .map_err(|e| {
                WorkRunsError::Provider(crate::models::providers::errors::IntegrationError::Other(
                    e.to_string(),
                ))
            })?;
        IntegrationClient::from_provider(&provider)
            .fetch_task(&run.external_task_ref)
            .await
            .map_err(WorkRunsError::from)
    }
}

#[must_use]
fn render_prompt_text(
    run: &WorkRun,
    cfg: &JobConfigFields,
    task: &IntegrationTask,
    repos: &[JobRepo],
) -> String {
    match run.work_type {
        WorkRunType::Implementation => render_implementation_prompt(cfg, task, repos),
        WorkRunType::PullRequestReview => render_review_prompt(run, cfg, task),
    }
}

#[must_use]
pub(crate) fn render_implementation_prompt(
    cfg: &JobConfigFields,
    task: &IntegrationTask,
    repos: &[JobRepo],
) -> String {
    let repo_urls = repos
        .iter()
        .map(|repo| repo.url.as_str())
        .collect::<Vec<&str>>()
        .join("\n");
    let repo_full_names = repos
        .iter()
        .map(|repo| repo.full_name.clone())
        .collect::<Vec<String>>();
    let repo_names = repo_full_names.join("\n");
    let repo_layout = repo_layout(&repo_full_names);
    let repo_url = repos.first().map(|repo| repo.url.as_str()).unwrap_or("");
    let mut prompt_text = render_template(
        &cfg.prompt_template,
        &TemplateVars {
            task_title: &task.title,
            task_body: task.description.as_deref().unwrap_or(""),
            repo_url,
            repo_urls: &repo_urls,
            repo_names: &repo_names,
            repo_layout: &repo_layout,
            review_target_pr_url: "",
        },
    );

    prompt_text.push_str(ENVIRONMENT_INSTRUCTION);

    if !repos.is_empty() {
        prompt_text.push_str(GITHUB_INSTRUCTION);
    }

    prompt_text
}

#[must_use]
fn render_review_prompt(run: &WorkRun, cfg: &JobConfigFields, task: &IntegrationTask) -> String {
    let repo_names = match run.review_target_repo_full_name.as_deref() {
        Some(repo) => repo.to_owned(),
        None => cfg.repo_full_names.join("\n"),
    };
    let repo_urls = match run.review_target_repo_full_name.as_deref() {
        Some(repo) => github_repo_url(repo),
        None => cfg.repo_urls.join("\n"),
    };
    let repo_full_names = match run.review_target_repo_full_name.as_ref() {
        Some(repo) => vec![repo.clone()],
        None => cfg.repo_full_names.clone(),
    };
    let repo_layout = repo_layout(&repo_full_names);

    let mut prompt_text = render_template(
        &cfg.review_prompt_template,
        &TemplateVars {
            task_title: &task.title,
            task_body: task.description.as_deref().unwrap_or(""),
            repo_url: &repo_urls,
            repo_urls: &repo_urls,
            repo_names: &repo_names,
            repo_layout: &repo_layout,
            review_target_pr_url: run.review_target_pr_url.as_deref().unwrap_or(""),
        },
    );
    prompt_text.push_str(ENVIRONMENT_INSTRUCTION);
    if !repo_full_names.is_empty() {
        prompt_text.push_str(REVIEW_GITHUB_INSTRUCTION);
    }

    prompt_text
}

#[must_use]
fn empty_task(run: &WorkRun, cfg: &JobConfigFields) -> IntegrationTask {
    IntegrationTask {
        id: run.external_task_ref.clone(),
        title: String::new(),
        project_id: cfg.external_project_id.clone(),
        description: None,
        status: String::new(),
        priority: String::new(),
        number: None,
        project_slug: None,
        assignee_name: None,
        created_at: String::new(),
        updated_at: None,
        labels: Vec::new(),
    }
}

#[must_use]
fn shared_work_type(work_type: WorkRunType) -> vulcanum_shared::api::wire::WorkRunType {
    match work_type {
        WorkRunType::Implementation => vulcanum_shared::api::wire::WorkRunType::Implementation,
        WorkRunType::PullRequestReview => {
            vulcanum_shared::api::wire::WorkRunType::PullRequestReview
        }
    }
}
