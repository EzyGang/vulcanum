use vulcanum_shared::api_types::SubmitResultRequest;
use vulcanum_shared::runtime::types::FinishStatus;

use crate::services::providers::client::IntegrationClient;
use crate::services::providers::model::IntegrationType;
use crate::services::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::services::work_runs::service::record_review::review_comment;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub(crate) async fn sync_kaneo_on_result(
        &self,
        run: &WorkRun,
        params: &SubmitResultRequest,
        status: WorkRunStatus,
        pr_urls: &[String],
    ) {
        let project_config = match self.project_configs.find_by_id(run.project_config_id).await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    project_config_id = %run.project_config_id,
                    work_run_id = %run.id,
                    error = %e,
                    "failed to look up project config",
                );
                return;
            }
        };

        let provider_id = match project_config.provider_id {
            Some(pid) => pid,
            None => {
                tracing::warn!(
                    project_config_id = %run.project_config_id,
                    work_run_id = %run.id,
                    "no provider configured for project config",
                );
                return;
            }
        };

        let provider = match self
            .providers_repo
            .find_by_id(&self.db, provider_id, run.team_id)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(
                    provider_id = %provider_id,
                    work_run_id = %run.id,
                    error = %e,
                    "failed to look up provider",
                );
                return;
            }
        };

        let client = match provider.provider_type {
            IntegrationType::Kaneo => {
                IntegrationClient::new_kaneo(provider.instance_url, provider.api_key)
            }
        };

        let is_review = matches!(run.work_type, WorkRunType::PullRequestReview);
        let is_blocked = matches!(params.finish_status, Some(FinishStatus::Blocked));

        if !is_blocked && !is_review {
            let new_column = match params.finish_status {
                Some(FinishStatus::Completed) => &project_config.target_column,
                Some(FinishStatus::Failed) => &project_config.pickup_column,
                None => match status {
                    WorkRunStatus::Completed => &project_config.target_column,
                    _ => &project_config.pickup_column,
                },
                Some(FinishStatus::Blocked) => &project_config.pickup_column,
            };

            if let Err(e) = client
                .update_task_status(&run.external_task_ref, new_column)
                .await
            {
                tracing::warn!(
                    task_ref = %run.external_task_ref,
                    column = %new_column,
                    error = %e,
                    "failed to update kaneo task status",
                );
            }
        }

        let comment = if is_review {
            review_comment(run, params)
        } else {
            match (
                params.finish_summary.as_deref(),
                params.finish_blocked_reason.as_deref(),
            ) {
                (Some(s), Some(r)) => format!("**Summary:** {s}\n**Blocked:** {r}"),
                (Some(s), None) => format!("**Summary:** {s}"),
                _ => format!("PR: {}", pr_urls.join(", ")),
            }
        };

        if let Err(e) = client.add_comment(&run.external_task_ref, &comment).await {
            tracing::warn!(
                task_ref = %run.external_task_ref,
                error = %e,
                "failed to add kaneo comment",
            );
        }
    }
}
