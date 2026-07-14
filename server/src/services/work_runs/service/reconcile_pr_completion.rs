use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::project_configs::model::ProjectConfig;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::TaskPr;
use crate::services::providers::client::IntegrationClient;
use crate::services::work_runs::service::lifecycle_labels::LifecycleLabelState;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub(crate) async fn reconcile_pull_request_completion(
        &self,
        installation_id: i64,
        repo_full_name: &str,
        pr_number: i64,
    ) -> Result<usize, WorkRunsError> {
        let targets = self
            .work_runs_repo
            .list_task_pr_targets_for_pull_request(
                &self.db,
                installation_id,
                repo_full_name,
                pr_number,
            )
            .await?;
        let mut moved = 0;

        for target in targets {
            if self
                .reconcile_task_pr_completion(target.project_config_id, &target.external_task_ref)
                .await?
            {
                moved += 1;
            }
        }

        Ok(moved)
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
            .task_prs_are_terminal(config.team_id, task_ref, &task_prs)
            .await?
        {
            return Ok(false);
        }

        self.move_task_to_done(&config, task_ref).await
    }

    pub(crate) async fn task_prs_are_terminal(
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
                Ok(state) if state.is_terminal() => (),
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
        let current = client.fetch_task(task_ref).await?;

        if current.status != config.review_column {
            tracing::debug!(
                task_ref,
                expected_column = %config.review_column,
                current_column = %current.status,
                "task moved before PR completion update",
            );
            return Ok(false);
        }

        if !self
            .set_lifecycle_label_for_task(
                config,
                &client,
                task_ref,
                LifecycleLabelState::Done,
                Some(&current.labels),
            )
            .await
        {
            tracing::warn!(task_ref, "failed to apply Done lifecycle label");
            return Err(WorkRunsError::LifecycleLabelUpdate);
        }

        client
            .update_task_status(task_ref, &config.done_column)
            .await?;
        Ok(true)
    }
}
