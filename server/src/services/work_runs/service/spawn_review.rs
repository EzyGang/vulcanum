use crate::db::work_runs::queries::prs::UpsertTaskPrParams;
use crate::db::work_runs::queries::InsertWorkRunParams;
use crate::models::work_runs::model::{TaskPr, WorkRun, WorkRunStatus, WorkRunType};
use crate::services::providers::client::IntegrationClient;
use crate::services::work_runs::service::WorkRunsService;
use crate::util::github::parse_github_pr_url;

const PR_BLOCK_START: &str = "<!-- vulcanum:prs:start -->";
const PR_BLOCK_END: &str = "<!-- vulcanum:prs:end -->";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ReviewSpawnOutcome {
    NoPullRequests,
    ReviewNeeded,
    ReviewRunning,
}

impl WorkRunsService {
    pub(crate) async fn attach_prs_and_spawn_reviews(
        &self,
        run: &WorkRun,
        pr_urls: &[String],
    ) -> ReviewSpawnOutcome {
        if pr_urls.is_empty() || !matches!(run.work_type, WorkRunType::Implementation) {
            return ReviewSpawnOutcome::NoPullRequests;
        }

        let project_config = match self.project_configs.find_by_id(run.project_config_id).await {
            Ok(config) => config,
            Err(e) => {
                tracing::warn!(work_run_id = %run.id, error = %e, "failed to load project config for review spawn");
                return ReviewSpawnOutcome::ReviewNeeded;
            }
        };
        let settings = match self
            .project_configs
            .effective_settings(&project_config)
            .await
        {
            Ok(settings) => settings,
            Err(e) => {
                tracing::warn!(work_run_id = %run.id, error = %e, "failed to load effective settings for review spawn");
                return ReviewSpawnOutcome::ReviewNeeded;
            }
        };

        let mut task_prs = Vec::new();
        for pr_url in pr_urls {
            match parse_github_pr_url(pr_url) {
                Some(pr) => {
                    let repo_full_name = pr.repo().full_name();
                    if !project_config.repo_full_names.contains(&repo_full_name) {
                        tracing::warn!(work_run_id = %run.id, pr_url = %pr_url, "skipping review for PR outside configured repos");
                        continue;
                    }
                    match self
                        .work_runs_repo
                        .upsert_task_pr(
                            &self.db,
                            UpsertTaskPrParams {
                                project_config_id: run.project_config_id,
                                external_task_ref: &run.external_task_ref,
                                pr_url: pr.url(),
                                repo_full_name: &repo_full_name,
                                pr_number: pr.number(),
                                source_work_run_id: run.id,
                            },
                        )
                        .await
                    {
                        Ok(task_pr) => task_prs.push(task_pr),
                        Err(e) => {
                            tracing::warn!(work_run_id = %run.id, pr_url = %pr_url, error = %e, "failed to upsert task PR")
                        }
                    }
                }
                None => {
                    tracing::warn!(work_run_id = %run.id, pr_url = %pr_url, "skipping invalid GitHub PR URL")
                }
            }
        }

        if task_prs.is_empty() {
            return ReviewSpawnOutcome::ReviewNeeded;
        }

        let cfg = project_config.job_fields(settings.clone());
        let task = match self.fetch_task_for_run(run, &cfg).await {
            Ok(task) => task,
            Err(e) => {
                tracing::warn!(work_run_id = %run.id, error = %e, "failed to fetch task data for review spawn");
                return ReviewSpawnOutcome::ReviewNeeded;
            }
        };

        self.update_task_pr_block(run, task.description.as_deref().unwrap_or(""), &task_prs)
            .await;

        if !settings.review_enabled {
            return ReviewSpawnOutcome::ReviewNeeded;
        }

        let mut review_running = false;

        for task_pr in &task_prs {
            let params = InsertWorkRunParams {
                team_id: run.team_id,
                external_task_ref: run.external_task_ref.clone(),
                project_config_id: run.project_config_id,
                repo_full_names: vec![task_pr.repo_full_name.clone()],
                status: WorkRunStatus::Pending,
                work_type: WorkRunType::PullRequestReview,
                parent_work_run_id: Some(run.id),
                review_target_pr_url: Some(task_pr.pr_url.clone()),
                review_target_repo_full_name: Some(task_pr.repo_full_name.clone()),
            };

            match self
                .work_runs_repo
                .insert_work_run_if_not_active(&self.db, params)
                .await
            {
                Ok(true) | Ok(false) => review_running = true,
                Err(e) => {
                    tracing::warn!(work_run_id = %run.id, pr_url = %task_pr.pr_url, error = %e, "failed to insert review run");
                }
            }
        }

        match review_running {
            true => ReviewSpawnOutcome::ReviewRunning,
            false => ReviewSpawnOutcome::ReviewNeeded,
        }
    }

    async fn update_task_pr_block(&self, run: &WorkRun, task_body: &str, task_prs: &[TaskPr]) {
        let project_config = match self.project_configs.find_by_id(run.project_config_id).await {
            Ok(config) => config,
            Err(_) => return,
        };
        let provider_id = match project_config.provider_id {
            Some(provider_id) => provider_id,
            None => return,
        };
        let provider = match self
            .providers_repo
            .find_by_id(&self.db, provider_id, run.team_id)
            .await
        {
            Ok(provider) => provider,
            Err(e) => {
                tracing::warn!(work_run_id = %run.id, error = %e, "failed to load provider for PR block update");
                return;
            }
        };
        let client = IntegrationClient::from_provider(&provider);
        let body = upsert_pr_block(
            task_body,
            &task_prs
                .iter()
                .map(|pr| pr.pr_url.clone())
                .collect::<Vec<String>>(),
        );
        if let Err(e) = client
            .update_task_description(&run.external_task_ref, &body)
            .await
        {
            tracing::warn!(work_run_id = %run.id, error = %e, "failed to update Kaneo PR block");
        }
    }
}

#[must_use]
pub(crate) fn upsert_pr_block(body: &str, pr_urls: &[String]) -> String {
    let block = format!(
        "{PR_BLOCK_START}\nVulcanum PRs:\n{}\n{PR_BLOCK_END}",
        pr_urls
            .iter()
            .map(|url| format!("- {url}"))
            .collect::<Vec<String>>()
            .join("\n")
    );

    match (body.find(PR_BLOCK_START), body.find(PR_BLOCK_END)) {
        (Some(start), Some(end)) if start < end => {
            let end = end + PR_BLOCK_END.len();
            format!("{}{}{}", &body[..start], block, &body[end..])
        }
        _ => match body.trim().is_empty() {
            true => block,
            false => format!("{}\n\n{}", body.trim_end(), block),
        },
    }
}
