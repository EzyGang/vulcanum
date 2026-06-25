use uuid::Uuid;
use vulcanum_shared::api_types::SubmitResultRequest;
use vulcanum_shared::runtime::types::FinishStatus;

use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::services::work_runs::repository::queries::SetResultParams;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn submit_result(
        &self,
        id: Uuid,
        worker_id: Uuid,
        params: SubmitResultRequest,
    ) -> Result<WorkRun, WorkRunsError> {
        let status = match params.finish_status {
            Some(FinishStatus::Completed) => WorkRunStatus::Completed,
            Some(FinishStatus::Failed) | Some(FinishStatus::Blocked) => WorkRunStatus::Failed,
            None => {
                if params.exit_code == 0 {
                    WorkRunStatus::Completed
                } else {
                    WorkRunStatus::Failed
                }
            }
        };
        let pr_urls = normalized_pr_urls(&params);

        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;

        if !matches!(run.status, WorkRunStatus::Running) {
            return Err(WorkRunsError::InvalidStatusTransition);
        }

        if run.worker_id != Some(worker_id) {
            return Err(WorkRunsError::NotOwned);
        }

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
                    finish_summary: params.finish_summary.as_deref(),
                    finish_blocked_reason: None,
                    finish_next_column: None,
                    review_url: params.review_url.as_deref(),
                    review_body: params.review_body.as_deref(),
                    review_already_exists: params.review_already_exists,
                },
            )
            .await?;

        self.work_runs_repo
            .replace_pr_urls(&mut *tx, id, &pr_urls)
            .await?;

        if let Err(e) = self
            .workers_repo
            .decrement_active_jobs(&mut *tx, worker_id)
            .await
        {
            tracing::warn!(
                error = %e,
                worker_id = %worker_id,
                "failed to decrement active_jobs"
            );
        }

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

                            if let Err(e) = self
                                .work_runs_repo
                                .reset_worker_active_jobs(&mut *tx, worker_id)
                                .await
                            {
                                tracing::warn!(
                                    error = %e,
                                    worker_id = %worker_id,
                                    "failed to reset worker active jobs on unhealthy transition"
                                );
                            }

                            if let Err(e) = self
                                .workers_repo
                                .reset_active_jobs_only(&mut *tx, worker_id)
                                .await
                            {
                                tracing::warn!(
                                    error = %e,
                                    worker_id = %worker_id,
                                    "failed to reset worker active_jobs on unhealthy transition"
                                );
                            }
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

        let defer_success_target = match (status, run.work_type) {
            (WorkRunStatus::Completed, WorkRunType::Implementation) => {
                self.attach_prs_and_spawn_reviews(&run, &pr_urls).await
            }
            _ => false,
        };

        self.sync_task_tracker_on_result(&run, &params, status, &pr_urls, defer_success_target)
            .await;

        if matches!(run.work_type, WorkRunType::PullRequestReview) {
            self.record_review_result(&run, &params).await;
        }

        Ok(updated)
    }
}

#[must_use]
fn normalized_pr_urls(params: &SubmitResultRequest) -> Vec<String> {
    params.pr_urls.clone()
}
