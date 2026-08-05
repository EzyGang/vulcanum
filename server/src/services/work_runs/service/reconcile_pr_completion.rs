use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::project_configs::model::ProjectConfig;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::TaskPr;
use crate::services::providers::client::IntegrationClient;
use crate::services::work_runs::service::WorkRunsService;
use crate::util::github::github_pr_url;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum TaskPrCompletionDisposition {
    Moved,
    AlreadyDone,
    Retryable(String),
    Terminal(String),
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct PullRequestReconciliation {
    pub matched: usize,
    pub moved: usize,
    pub already_done: usize,
    pub retryable: Vec<String>,
    pub terminal: Vec<String>,
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
        let mut outcome = PullRequestReconciliation {
            matched: targets.len(),
            ..PullRequestReconciliation::default()
        };

        for target in targets {
            match self
                .reconcile_task_pr_completion_with_disposition(
                    target.project_config_id,
                    &target.external_task_ref,
                )
                .await?
            {
                TaskPrCompletionDisposition::Moved => outcome.moved += 1,
                TaskPrCompletionDisposition::AlreadyDone => outcome.already_done += 1,
                TaskPrCompletionDisposition::Retryable(reason) => outcome.retryable.push(reason),
                TaskPrCompletionDisposition::Terminal(reason) => outcome.terminal.push(reason),
            }
        }

        Ok(outcome)
    }

    pub(crate) async fn reconcile_task_pr_completion(
        &self,
        project_config_id: uuid::Uuid,
        task_ref: &str,
    ) -> Result<bool, WorkRunsError> {
        Ok(matches!(
            self.reconcile_task_pr_completion_with_disposition(project_config_id, task_ref)
                .await?,
            TaskPrCompletionDisposition::Moved
        ))
    }

    async fn reconcile_task_pr_completion_with_disposition(
        &self,
        project_config_id: uuid::Uuid,
        task_ref: &str,
    ) -> Result<TaskPrCompletionDisposition, WorkRunsError> {
        let config = self.project_configs.find_by_id(project_config_id).await?;
        if !config.enabled {
            return Ok(TaskPrCompletionDisposition::Terminal(format!(
                "task {task_ref} belongs to a disabled project configuration"
            )));
        }
        if config.review_column == config.done_column {
            return Ok(TaskPrCompletionDisposition::Terminal(format!(
                "task {task_ref} has identical Review and Done columns"
            )));
        }

        let mut transaction = self.db.begin().await?;
        self.work_runs_repo
            .lock_task_pr_completion(&mut transaction, config.id, task_ref)
            .await?;
        let task_refs = [task_ref.to_owned()];
        let task_prs = self
            .work_runs_repo
            .list_task_prs_for_refs(&mut *transaction, config.id, &task_refs)
            .await?;
        if !self
            .task_prs_are_merged(config.team_id, task_ref, &task_prs)
            .await?
        {
            return Ok(TaskPrCompletionDisposition::Retryable(format!(
                "task {task_ref} is waiting for all linked pull requests to merge"
            )));
        }

        let disposition = self
            .move_task_to_done_with_disposition(&config, task_ref)
            .await?;
        transaction.commit().await?;

        Ok(disposition)
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

    async fn move_task_to_done_with_disposition(
        &self,
        config: &ProjectConfig,
        task_ref: &str,
    ) -> Result<TaskPrCompletionDisposition, WorkRunsError> {
        let provider_id = config.provider_id.ok_or(ProjectConfigsError::NoProvider)?;
        let provider = self
            .providers_repo
            .find_by_id(&self.db, provider_id, config.team_id)
            .await
            .map_err(|_| ProjectConfigsError::NoProvider)?;
        let client = IntegrationClient::from_provider(&provider);
        self.sync_task_to_done_with_disposition(config, task_ref, &client)
            .await
    }

    #[cfg(test)]
    pub(crate) async fn sync_task_to_done(
        &self,
        config: &ProjectConfig,
        task_ref: &str,
        client: &IntegrationClient,
    ) -> Result<bool, WorkRunsError> {
        Ok(matches!(
            self.sync_task_to_done_with_disposition(config, task_ref, client)
                .await?,
            TaskPrCompletionDisposition::Moved
        ))
    }

    async fn sync_task_to_done_with_disposition(
        &self,
        config: &ProjectConfig,
        task_ref: &str,
        client: &IntegrationClient,
    ) -> Result<TaskPrCompletionDisposition, WorkRunsError> {
        let current = match &self.task_fetcher {
            Some(task_fetcher) => task_fetcher.fetch_task(task_ref).await?,
            None => client.fetch_task(task_ref).await?,
        };

        if current.status == config.done_column {
            return Ok(TaskPrCompletionDisposition::AlreadyDone);
        }
        if current.status != config.review_column {
            return Ok(TaskPrCompletionDisposition::Terminal(format!(
                "task {task_ref} is in {} instead of the configured Review or Done column",
                current.status
            )));
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
        Ok(TaskPrCompletionDisposition::Moved)
    }
}
