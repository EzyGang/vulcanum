use uuid::Uuid;

use crate::services::providers::client::IntegrationClient;
use crate::services::providers::model::IntegrationType;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::{WorkRun, WorkRunStatus};
use crate::services::work_runs::repository::queries::SetResultParams;
use crate::services::work_runs::service::WorkRunsService;
use vulcanum_shared::api_types::SubmitResultRequest;
use vulcanum_shared::runtime::types::FinishStatus;

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
                    finish_blocked_reason: params.finish_blocked_reason.as_deref(),
                    finish_next_column: params.finish_next_column.as_deref(),
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

        self.sync_kaneo_on_result(&run, &params, status, &pr_urls)
            .await;

        Ok(updated)
    }

    async fn sync_kaneo_on_result(
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

        let is_blocked = matches!(params.finish_status, Some(FinishStatus::Blocked));

        if !is_blocked {
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

        let comment = match (
            params.finish_summary.as_deref(),
            params.finish_blocked_reason.as_deref(),
        ) {
            (Some(s), Some(r)) => format!("**Summary:** {s}\n**Blocked:** {r}"),
            (Some(s), None) => format!("**Summary:** {s}"),
            _ => format!("PR: {}", pr_urls.join(", ")),
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

fn normalized_pr_urls(params: &SubmitResultRequest) -> Vec<String> {
    if !params.pr_urls.is_empty() {
        return params.pr_urls.clone();
    }

    match params.pr_url.is_empty() {
        true => Vec::new(),
        false => vec![params.pr_url.clone()],
    }
}
