use vulcanum_shared::api_types::SubmitResultRequest;
use vulcanum_shared::review_feedback::review_requires_implementation;
use vulcanum_shared::runtime::types::FinishStatus;

use crate::services::providers::client::IntegrationClient;
use crate::services::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::services::work_runs::service::record_review::review_comment;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub(crate) async fn sync_task_tracker_on_result(
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

        let client = IntegrationClient::from_provider(provider);
        let is_review = matches!(run.work_type, WorkRunType::PullRequestReview);
        let is_blocked = matches!(params.finish_status, Some(FinishStatus::Blocked));

        let result_column = match (is_blocked, is_review) {
            (true, _) => None,
            (false, true) => review_result_column(
                params.review_already_exists,
                params.review_body.as_deref(),
                &project_config.progress_column,
            ),
            (false, false) => Some(implementation_result_column(
                params.finish_status,
                status,
                &project_config.pickup_column,
                &project_config.target_column,
            )),
        };

        if let Some(new_column) = result_column {
            if let Err(e) = client
                .update_task_status(&run.external_task_ref, new_column)
                .await
            {
                tracing::warn!(
                    task_ref = %run.external_task_ref,
                    column = %new_column,
                    error = %e,
                    "failed to update task status",
                );
            }
        }

        let comment = if is_review {
            review_comment(run, params)
        } else {
            params
                .finish_summary
                .as_ref()
                .map(|summary| format!("**Summary:** {summary}"))
                .unwrap_or_else(|| format!("PR: {}", pr_urls.join(", ")))
        };

        if let Err(e) = client.add_comment(&run.external_task_ref, &comment).await {
            tracing::warn!(
                task_ref = %run.external_task_ref,
                error = %e,
                "failed to add task comment",
            );
        }
    }
}

#[must_use]
pub(crate) fn review_result_column<'a>(
    review_already_exists: bool,
    review_body: Option<&str>,
    progress_column: &'a str,
) -> Option<&'a str> {
    if review_already_exists {
        return None;
    }

    match review_body {
        Some(body) if review_requires_implementation(body) => Some(progress_column),
        Some(_) | None => None,
    }
}

#[must_use]
pub(crate) fn implementation_result_column<'a>(
    finish_status: Option<FinishStatus>,
    run_status: WorkRunStatus,
    pickup_column: &'a str,
    target_column: &'a str,
) -> &'a str {
    match finish_status {
        Some(FinishStatus::Completed) => target_column,
        Some(FinishStatus::Failed) | Some(FinishStatus::Blocked) => pickup_column,
        None => match run_status {
            WorkRunStatus::Completed => target_column,
            _ => pickup_column,
        },
    }
}
