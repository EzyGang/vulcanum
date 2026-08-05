use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::project_configs::model::ProjectConfig;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::TaskPr;
use crate::services::providers::client::IntegrationClient;
use crate::services::work_runs::service::WorkRunsService;
use crate::util::github::github_pr_url;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct PullRequestReconciliation {
    pub matched: usize,
    pub moved: usize,
}

impl WorkRunsService {
    pub(crate) async fn reconcile_pull_request_completion(
        &self,
        repo_full_name: &str,
        pr_number: i64,
    ) -> Result<PullRequestReconciliation, WorkRunsError> {
        let pr_url = github_pr_url(repo_full_name, pr_number);
        let targets = self
            .work_runs_repo
            .list_task_pr_targets_for_pr_url(&self.db, &pr_url)
            .await?;
        let matched = targets.len();
        let mut moved = 0;

        for target in targets {
            if self
                .reconcile_task_pr_completion(target.project_config_id, &target.external_task_ref)
                .await?
            {
                moved += 1;
            }
        }

        Ok(PullRequestReconciliation { matched, moved })
    }

    pub(crate) async fn reconcile_task_pr_completion(
        &self,
        project_config_id: uuid::Uuid,
        task_ref: &str,
    ) -> Result<bool, WorkRunsError> {
        let config = self.project_configs.find_by_id(project_config_id).await?;
        if !config.enabled || config.review_column == config.done_column {
            return Ok(false);
        }

        let task_refs = [task_ref.to_owned()];
        let task_prs = self
            .work_runs_repo
            .list_task_prs_for_refs(&self.db, config.id, &task_refs)
            .await?;
        if !self
            .task_prs_are_merged(config.team_id, task_ref, &task_prs)
            .await?
        {
            return Ok(false);
        }

        self.move_task_to_done(&config, task_ref).await
    }

    pub(crate) async fn task_prs_are_merged(
        &self,
        team_id: uuid::Uuid,
        task_ref: &str,
        task_prs: &[TaskPr],
    ) -> Result<bool, WorkRunsError> {
        for task_pr in task_prs {
            match self
                .pr_state_reader
                .pull_request_state(team_id, &task_pr.repo_full_name, task_pr.pr_number)
                .await
            {
                Ok(
                    crate::services::github_app::service::pull_requests::PullRequestState::Merged,
                ) => (),
                Ok(_) => return Ok(false),
                Err(e) => {
                    tracing::warn!(
                        task_ref,
                        pr_url = %task_pr.pr_url,
                        error = %e,
                        "failed to read pull request state",
                    );
                    return Err(e.into());
                }
            }
        }

        Ok(!task_prs.is_empty())
    }

    async fn move_task_to_done(
        &self,
        config: &ProjectConfig,
        task_ref: &str,
    ) -> Result<bool, WorkRunsError> {
        let provider_id = config.provider_id.ok_or(ProjectConfigsError::NoProvider)?;
        let provider = self
            .providers_repo
            .find_by_id(&self.db, provider_id, config.team_id)
            .await
            .map_err(|_| ProjectConfigsError::NoProvider)?;
        let client = IntegrationClient::from_provider(&provider);
        self.sync_task_to_done(config, task_ref, &client).await
    }

    pub(crate) async fn sync_task_to_done(
        &self,
        config: &ProjectConfig,
        task_ref: &str,
        client: &IntegrationClient,
    ) -> Result<bool, WorkRunsError> {
        let current = match &self.task_fetcher {
            Some(task_fetcher) => task_fetcher.fetch_task(task_ref).await?,
            None => client.fetch_task(task_ref).await?,
        };

        if current.status != config.review_column {
            tracing::debug!(
                task_ref,
                expected_column = %config.review_column,
                current_column = %current.status,
                "skipping automatic PR completion because task left the Review column",
            );
            return Ok(false);
        }

        match &self.task_fetcher {
            Some(task_fetcher) => {
                task_fetcher
                    .update_task_status(task_ref, &config.done_column)
                    .await?;
            }
            None => {
                client
                    .update_task_status(task_ref, &config.done_column)
                    .await?
            }
        }

        if !self
            .clear_lifecycle_labels_for_task(client, task_ref, Some(&current.labels))
            .await
        {
            tracing::warn!(
                task_ref,
                "task moved to Done without removing its lifecycle labels"
            );
        }
        Ok(true)
    }
}
