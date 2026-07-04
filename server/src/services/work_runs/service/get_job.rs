use uuid::Uuid;
use vulcanum_shared::api_types::JobRepo;
use vulcanum_shared::api_types::JobResponse;

use crate::models::project_configs::model::JobConfigFields;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::WorkRunType;
use crate::services::model_providers::renderer::ModelSelection;
use crate::services::poller::prompts::{ENVIRONMENT_INSTRUCTION, GITHUB_INSTRUCTION};
use crate::services::poller::service::repo_layout;
use crate::services::poller::template::{render_template, TemplateVars};
use crate::services::work_runs::service::task_data::TaskRenderData;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn get_job(&self, id: Uuid, worker_id: Uuid) -> Result<JobResponse, WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;
        if run.worker_id != Some(worker_id) {
            return Err(WorkRunsError::NotOwned);
        }

        let (cfg, task_data) = match self.project_configs.find_by_id(run.project_config_id).await {
            Ok(config) => {
                let settings = self.project_configs.effective_settings(&config).await?;
                let task_data = self
                    .fetch_task_render_data(&config, &run.external_task_ref)
                    .await?;
                (config.job_fields(settings), task_data)
            }
            Err(_) => {
                tracing::warn!(
                    project_config_id = %run.project_config_id,
                    work_run_id = %id,
                    "project config not found for work run"
                );
                (
                    JobConfigFields::empty_for_team(run.team_id),
                    TaskRenderData::default(),
                )
            }
        };
        let repos = self.github_repos_for_work_run(&run).await?;
        let pr_urls = self.work_runs_repo.list_pr_urls(&self.db, id).await?;
        let prompt_text = render_job_prompt(
            &cfg,
            run.work_type,
            &task_data,
            &repos,
            run.review_target_pr_url.as_deref().unwrap_or(""),
        );

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

#[must_use]
fn render_job_prompt(
    cfg: &JobConfigFields,
    work_type: WorkRunType,
    task_data: &TaskRenderData,
    repos: &[JobRepo],
    review_target_pr_url: &str,
) -> String {
    let repo_urls = repos
        .iter()
        .map(|repo| repo.url.as_str())
        .collect::<Vec<&str>>()
        .join("\n");
    let repo_names = repos
        .iter()
        .map(|repo| repo.full_name.as_str())
        .collect::<Vec<&str>>()
        .join("\n");
    let full_names = repos
        .iter()
        .map(|repo| repo.full_name.clone())
        .collect::<Vec<String>>();
    let repo_layout = repo_layout(&full_names);
    let template = match work_type {
        WorkRunType::Implementation => cfg.prompt_template.as_str(),
        WorkRunType::PullRequestReview => cfg.review_prompt_template.as_str(),
    };
    let mut prompt_text = render_template(
        template,
        &TemplateVars {
            task_title: &task_data.title,
            task_body: &task_data.body,
            repo_url: repos.first().map(|repo| repo.url.as_str()).unwrap_or(""),
            repo_urls: &repo_urls,
            repo_names: &repo_names,
            repo_layout: &repo_layout,
            review_target_pr_url,
        },
    );

    if matches!(work_type, WorkRunType::Implementation) {
        prompt_text.push_str(ENVIRONMENT_INSTRUCTION);
        if !repos.is_empty() {
            prompt_text.push_str(GITHUB_INSTRUCTION);
        }
    }

    prompt_text
}
