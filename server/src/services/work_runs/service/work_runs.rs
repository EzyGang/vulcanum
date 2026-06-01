use uuid::Uuid;

use crate::services::integrations::client::IntegrationClient;
use crate::services::integrations::model::IntegrationType;
use crate::services::project_configs::model::JobConfigFields;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::{WorkRun, WorkRunListItem, WorkRunStatus};
use crate::services::work_runs::repository::work_runs::SetResultParams;
use crate::services::work_runs::service::WorkRunsService;
use vulcanum_shared::api_types::{JobResponse, SubmitResultRequest};

impl WorkRunsService {
    pub async fn poll(&self, worker_id: Uuid) -> Result<Option<Uuid>, WorkRunsError> {
        if let Err(e) = self
            .workers_repo
            .update_last_seen(&self.db, worker_id, chrono::Utc::now())
            .await
        {
            tracing::warn!(error = %e, worker_id = %worker_id, "failed to update last_seen");
        }

        let dispatched_id = self.dispatch_store.take_dispatched(worker_id).await?;

        Ok(dispatched_id)
    }

    pub async fn get_job(&self, id: Uuid) -> Result<JobResponse, WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;

        let config = self
            .project_configs_repo
            .find_by_id(&self.db, run.project_config_id)
            .await;

        let cfg = match config {
            Ok(ref c) => c.job_fields(),
            Err(_) => {
                tracing::warn!(
                    project_config_id = %run.project_config_id,
                    work_run_id = %id,
                    "project config not found for work run"
                );
                JobConfigFields::default()
            }
        };

        let (kaneo_instance, kaneo_api_key) = match cfg.provider_id {
            Some(pid) => match self.providers_repo.find_by_id(&self.db, pid).await {
                Ok(provider) => (provider.instance_url, provider.api_key),
                Err(_) => (String::new(), String::new()),
            },
            None => (String::new(), String::new()),
        };

        Ok(JobResponse {
            prompt_text: run.prompt_text,
            repo_url: run.repo_url,
            agents_md: run.agents_md,
            opencode_config: cfg.opencode_config,
            external_task_ref: run.external_task_ref,
            kaneo_instance,
            kaneo_api_key,
            kaneo_project_id: cfg.kaneo_project_id,
            kaneo_workspace_id: cfg.kaneo_workspace_id,
        })
    }

    pub async fn ack_job(&self, id: Uuid, worker_id: Uuid) -> Result<WorkRun, WorkRunsError> {
        self.work_runs_repo
            .acknowledge(&self.db, id, worker_id)
            .await
    }

    pub async fn list_all(
        &self,
        status: Option<WorkRunStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WorkRunListItem>, WorkRunsError> {
        self.work_runs_repo
            .list_all(&self.db, status, limit, offset)
            .await
    }

    pub async fn delete_run(&self, id: Uuid) -> Result<(), WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;

        if matches!(run.status, WorkRunStatus::Running) {
            return Err(WorkRunsError::DeleteRunning);
        }

        let mut tx = self.db.begin().await.map_err(WorkRunsError::Database)?;

        if let Some(worker_id) = run.worker_id {
            if matches!(run.status, WorkRunStatus::Dispatched) {
                if let Err(e) = self
                    .workers_repo
                    .decrement_active_jobs(&mut *tx, worker_id)
                    .await
                {
                    tracing::warn!(
                        error = %e,
                        worker_id = %worker_id,
                        work_run_id = %id,
                        "failed to decrement active_jobs on run deletion"
                    );
                }
            }
        }

        let delete_r = self.work_runs_repo.delete(&mut *tx, id).await;
        if let Err(e) = delete_r {
            let _ = tx.rollback().await;
            return Err(e);
        }

        tx.commit().await.map_err(WorkRunsError::Database)
    }

    pub async fn fail_run(&self, id: Uuid) -> Result<WorkRun, WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;

        match run.status {
            WorkRunStatus::Running | WorkRunStatus::Dispatched => (),
            _ => return Err(WorkRunsError::InvalidStatusTransition),
        }

        let mut tx = self.db.begin().await.map_err(WorkRunsError::Database)?;

        let updated = self
            .work_runs_repo
            .force_fail(&mut *tx, id)
            .await?
            .ok_or(WorkRunsError::NotFound)?;

        if let Some(worker_id) = updated.worker_id {
            if let Err(e) = self
                .workers_repo
                .decrement_active_jobs(&mut *tx, worker_id)
                .await
            {
                tracing::warn!(
                    error = %e,
                    worker_id = %worker_id,
                    "failed to decrement active_jobs on force fail"
                );
            }
        }

        tx.commit().await.map_err(WorkRunsError::Database)?;

        self.clear_cancel_flag(id).await;

        Ok(updated)
    }

    pub async fn cancel_run(&self, id: Uuid) -> Result<(), WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;

        match run.status {
            WorkRunStatus::Running | WorkRunStatus::Dispatched => (),
            _ => return Err(WorkRunsError::InvalidStatusTransition),
        }

        self.cancel_store
            .request_cancel(id)
            .await
            .map_err(WorkRunsError::Dispatch)?;

        Ok(())
    }

    pub async fn bulk_delete_runs(&self, ids: &[Uuid]) -> Result<u64, WorkRunsError> {
        let mut tx = self.db.begin().await.map_err(WorkRunsError::Database)?;
        let mut deleted = 0u64;

        for id in ids {
            match self.work_runs_repo.find_by_id(&mut *tx, *id).await {
                Ok(run) => {
                    if matches!(run.status, WorkRunStatus::Running) {
                        tracing::warn!(work_run_id = %id, "skipping running run in bulk delete");
                        continue;
                    }

                    if let Some(worker_id) = run.worker_id {
                        if matches!(run.status, WorkRunStatus::Dispatched) {
                            if let Err(e) = self
                                .workers_repo
                                .decrement_active_jobs(&mut *tx, worker_id)
                                .await
                            {
                                tracing::warn!(
                                    error = %e,
                                    worker_id = %worker_id,
                                    work_run_id = %id,
                                    "failed to decrement active_jobs on bulk delete"
                                );
                            }
                        }
                    }

                    self.work_runs_repo.delete(&mut *tx, *id).await?;

                    deleted += 1;
                }
                Err(e) => {
                    tracing::warn!(work_run_id = %id, error = %e, "skipping not found run in bulk delete");
                }
            }
        }

        tx.commit().await.map_err(WorkRunsError::Database)?;

        Ok(deleted)
    }

    pub async fn submit_result(
        &self,
        id: Uuid,
        worker_id: Uuid,
        params: SubmitResultRequest,
    ) -> Result<WorkRun, WorkRunsError> {
        let status = if params.exit_code == 0 {
            WorkRunStatus::Completed
        } else {
            WorkRunStatus::Failed
        };

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
                    pr_url: &params.pr_url,
                    exit_code: params.exit_code,
                    tokens_used: params.tokens_used,
                    duration_ms: params.duration_ms,
                    status,
                    input_tokens: params.input_tokens,
                    output_tokens: params.output_tokens,
                    cache_read_tokens: params.cache_read_tokens,
                    cache_write_tokens: params.cache_write_tokens,
                    model_used: params.model_used.as_deref(),
                },
            )
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
            duration_ms = params.duration_ms,
            exit_code = params.exit_code,
            has_pr_url = !params.pr_url.is_empty(),
            "work_run completed by worker",
        );

        self.sync_kaneo_on_result(&run, &params, status).await;

        Ok(updated)
    }

    async fn clear_cancel_flag(&self, work_run_id: Uuid) {
        if let Err(e) = self.cancel_store.take_cancel(work_run_id).await {
            tracing::warn!(
                error = %e,
                work_run_id = %work_run_id,
                "failed to clear cancel flag on terminal status"
            );
        }
    }

    async fn sync_kaneo_on_result(
        &self,
        run: &WorkRun,
        params: &SubmitResultRequest,
        status: WorkRunStatus,
    ) {
        let project_config = match self
            .project_configs_repo
            .find_by_id(&self.db, run.project_config_id)
            .await
        {
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

        let provider = match self.providers_repo.find_by_id(&self.db, provider_id).await {
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

        let new_column = match status {
            WorkRunStatus::Completed => &project_config.target_column,
            _ => &project_config.pickup_column,
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

        if let Err(e) = client
            .add_comment(&run.external_task_ref, &format!("PR: {}", params.pr_url))
            .await
        {
            tracing::warn!(
                task_ref = %run.external_task_ref,
                error = %e,
                "failed to add kaneo comment",
            );
        }
    }
}
