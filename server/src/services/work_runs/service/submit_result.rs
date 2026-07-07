use uuid::Uuid;
use vulcanum_shared::api_types::SubmitResultRequest;
use vulcanum_shared::runtime::types::FinishStatus;

use crate::db::task_augmentations::IncrementTaskUsageParams;
use crate::db::work_runs::queries::SetResultParams;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::services::work_runs::service::spawn_review::ReviewSpawnOutcome;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn submit_result(
        &self,
        id: Uuid,
        worker_id: Uuid,
        params: SubmitResultRequest,
    ) -> Result<WorkRun, WorkRunsError> {
        let status = result_status(&params);
        let pr_urls = normalized_pr_urls(&params);

        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;

        if !matches!(run.status, WorkRunStatus::Running) {
            if is_idempotent_retry(&run, worker_id, &params, status, &pr_urls) {
                let existing_pr_urls = self.work_runs_repo.list_pr_urls(&self.db, id).await?;
                if existing_pr_urls == pr_urls {
                    return Ok(run);
                }
            }

            return Err(WorkRunsError::InvalidStatusTransition);
        }

        if run.worker_id != Some(worker_id) {
            return Err(WorkRunsError::NotOwned);
        }

        let finish_blocked_reason = blocked_finish_reason(&params);

        let mut tx = self.db.begin().await.map_err(WorkRunsError::Database)?;

        let updated = self
            .work_runs_repo
            .set_result(
                &mut *tx,
                id,
                SetResultParams {
                    pr_url: pr_urls.first().map(String::as_str).unwrap_or(""),
                    exit_code: params.exit_code,
                    tokens_used: params.tokens_used,
                    duration_ms: params.duration_ms,
                    status,
                    input_tokens: params.input_tokens,
                    output_tokens: params.output_tokens,
                    cache_read_tokens: params.cache_read_tokens,
                    cache_write_tokens: params.cache_write_tokens,
                    model_used: params.model_used.as_deref(),
                    finish_status: params.finish_status.as_ref().map(|s| s.as_str()),
                    result_summary: params.result_summary.as_deref(),
                    finish_blocked_reason,
                    finish_next_column: None,
                },
            )
            .await?;

        self.work_runs_repo
            .replace_pr_urls(&mut *tx, id, &pr_urls)
            .await?;

        self.task_augmentations_repo
            .increment_usage(
                &mut *tx,
                IncrementTaskUsageParams {
                    team_id: run.team_id,
                    project_config_id: run.project_config_id,
                    external_task_ref: &run.external_task_ref,
                    tokens_used: params.tokens_used,
                    input_tokens: params.input_tokens,
                    output_tokens: params.output_tokens,
                    cache_read_tokens: params.cache_read_tokens,
                    cache_write_tokens: params.cache_write_tokens,
                },
            )
            .await?;

        self.workers_repo
            .decrement_active_jobs(&mut *tx, worker_id)
            .await?;

        match status {
            WorkRunStatus::Completed => {
                if let Err(e) = self
                    .workers_repo
                    .reset_consecutive_errors(&mut *tx, worker_id)
                    .await
                {
                    tracing::warn!(
                        error = %e,
                        worker_id = %worker_id,
                        "failed to reset consecutive errors"
                    );
                }
            }
            WorkRunStatus::Failed => {
                match self
                    .workers_repo
                    .increment_consecutive_errors(&mut *tx, worker_id, self.unhealthy_threshold)
                    .await
                {
                    Ok(consecutive_errors) => {
                        if consecutive_errors >= self.unhealthy_threshold {
                            tracing::warn!(
                                worker_id = %worker_id,
                                consecutive_errors,
                                threshold = self.unhealthy_threshold,
                                "worker reached unhealthy threshold, marking unhealthy"
                            );

                            self.work_runs_repo
                                .reset_worker_active_jobs(&mut *tx, worker_id)
                                .await?;

                            self.workers_repo
                                .reset_active_jobs_only(&mut *tx, worker_id)
                                .await?;
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            worker_id = %worker_id,
                            "failed to increment consecutive errors"
                        );
                    }
                }
            }
            _ => (),
        }

        tx.commit().await.map_err(WorkRunsError::Database)?;

        self.clear_cancel_flag(id).await;

        tracing::info!(
            worker_id = %worker_id,
            work_run_id = %id,
            tokens_used = params.tokens_used,
            input_tokens = params.input_tokens,
            output_tokens = params.output_tokens,
            cache_read_tokens = params.cache_read_tokens,
            cache_write_tokens = params.cache_write_tokens,
            model_used = params.model_used.as_deref(),
            duration_ms = params.duration_ms,
            exit_code = params.exit_code,
            has_pr_url = !pr_urls.is_empty(),
            "work_run completed by worker",
        );

        let review_outcome = match (status, run.work_type) {
            (WorkRunStatus::Completed, WorkRunType::Implementation) => {
                Some(self.attach_prs_and_spawn_reviews(&run, &pr_urls).await)
            }
            _ => None,
        };

        self.sync_task_tracker_on_result(
            &run,
            &params,
            status,
            &pr_urls,
            matches!(review_outcome, Some(ReviewSpawnOutcome::ReviewRunning)),
        )
        .await;

        if matches!(run.work_type, WorkRunType::PullRequestReview) {
            self.record_review_result(&run, &params).await;
        }

        self.set_lifecycle_label_after_result(&run, status, review_outcome)
            .await;

        Ok(updated)
    }
}

#[must_use]
fn result_status(params: &SubmitResultRequest) -> WorkRunStatus {
    match params.finish_status {
        Some(FinishStatus::Completed) => WorkRunStatus::Completed,
        Some(FinishStatus::Failed | FinishStatus::Blocked) => WorkRunStatus::Failed,
        None => match params.exit_code {
            0 => WorkRunStatus::Completed,
            _ => WorkRunStatus::Failed,
        },
    }
}

#[must_use]
fn blocked_finish_reason(params: &SubmitResultRequest) -> Option<&str> {
    match params.finish_status {
        Some(FinishStatus::Blocked) => params.result_summary.as_deref().or(Some("blocked")),
        _ => None,
    }
}

#[must_use]
fn is_idempotent_retry(
    run: &WorkRun,
    worker_id: Uuid,
    params: &SubmitResultRequest,
    status: WorkRunStatus,
    pr_urls: &[String],
) -> bool {
    if run.worker_id != Some(worker_id) {
        return false;
    }

    if !matches!(run.status, WorkRunStatus::Completed | WorkRunStatus::Failed) {
        return false;
    }

    run.status == status
        && run.result_pr_url.as_deref() == Some(pr_urls.first().map(String::as_str).unwrap_or(""))
        && run.result_exit_code == Some(params.exit_code)
        && run.tokens_used == Some(params.tokens_used)
        && run.duration_ms == Some(params.duration_ms)
        && run.input_tokens == Some(params.input_tokens)
        && run.output_tokens == Some(params.output_tokens)
        && run.cache_read_tokens == Some(params.cache_read_tokens)
        && run.cache_write_tokens == Some(params.cache_write_tokens)
        && run.model_used.as_deref() == params.model_used.as_deref()
        && run.finish_status.as_deref() == params.finish_status.as_ref().map(|s| s.as_str())
        && run.result_summary.as_deref() == params.result_summary.as_deref()
        && run.finish_next_column.is_none()
        && run.finish_blocked_reason.as_deref() == blocked_finish_reason(params)
}

#[must_use]
fn normalized_pr_urls(params: &SubmitResultRequest) -> Vec<String> {
    params.pr_urls.clone()
}
