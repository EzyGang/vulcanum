use chrono::Utc;
use uuid::Uuid;

use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::{WorkRun, WorkRunListItem, WorkRunStatus};
use crate::services::work_runs::repository::work_runs::SetResultParams;
use crate::services::work_runs::service::WorkRunsService;
use vulcanum_shared::api_types::SubmitResultRequest;

impl WorkRunsService {
    pub async fn poll(&self, worker_id: Uuid) -> Result<Option<Uuid>, WorkRunsError> {
        if let Err(e) = self
            .workers_repo
            .update_last_seen(&self.db, worker_id, Utc::now())
            .await
        {
            tracing::warn!("Failed to update last_seen for worker {}: {}", worker_id, e);
        }

        if let Err(e) = self
            .workers_repo
            .mark_stale_disconnected(&self.db, self.stale_threshold)
            .await
        {
            tracing::warn!("Failed to mark stale workers: {}", e);
        }

        self.notifier.add_worker(worker_id).await;

        let work_run_id = self.notifier.take(&worker_id).await;

        if !work_run_id {
            return Ok(None);
        }

        let run_id = self.work_runs_repo.find_oldest_pending_id(&self.db).await?;

        if let Some((id, external_task_ref)) = run_id {
            tracing::info!(
                worker_id = worker_id.to_string().as_str(),
                work_run_id = id.to_string().as_str(),
                external_task_ref = external_task_ref.as_str(),
                "dispatching work_run {} to worker {}",
                id,
                worker_id,
            );
            return Ok(Some(id));
        }

        Ok(None)
    }

    pub async fn get_job(&self, id: Uuid) -> Result<WorkRun, WorkRunsError> {
        self.work_runs_repo.find_by_id(&self.db, id).await
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

        let updated = self
            .work_runs_repo
            .set_result(
                &self.db,
                id,
                SetResultParams {
                    pr_url: &params.pr_url,
                    exit_code: params.exit_code,
                    tokens_used: params.tokens_used,
                    duration_ms: params.duration_ms,
                    status,
                },
            )
            .await?;

        tracing::info!(
            worker_id = worker_id.to_string().as_str(),
            work_run_id = id.to_string().as_str(),
            tokens_used = params.tokens_used,
            duration_ms = params.duration_ms,
            exit_code = params.exit_code,
            has_pr_url = !params.pr_url.is_empty(),
            "work_run {} completed by worker {}",
            id,
            worker_id,
        );

        self.sync_kaneo_on_result(&run, &params, status).await;

        Ok(updated)
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
                    "Failed to look up project config {} for work_run {}: {}",
                    run.project_config_id,
                    run.id,
                    e,
                );
                return;
            }
        };

        let new_column = match status {
            WorkRunStatus::Completed => &project_config.target_column,
            _ => &project_config.pickup_column,
        };

        if let Err(e) = self
            .kaneo
            .update_task_status(&run.external_task_ref, new_column)
            .await
        {
            tracing::warn!(
                "Failed to update kaneo task {} status to {}: {}",
                run.external_task_ref,
                new_column,
                e,
            );
        }

        if let Err(e) = self
            .kaneo
            .add_comment(&run.external_task_ref, &format!("PR: {}", params.pr_url))
            .await
        {
            tracing::warn!(
                "Failed to add kaneo comment for task {}: {}",
                run.external_task_ref,
                e,
            );
        }
    }
}
