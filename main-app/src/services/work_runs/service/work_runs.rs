use chrono::Utc;
use uuid::Uuid;

use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::{WorkRun, WorkRunStatus};
use crate::services::work_runs::repository::work_runs::SetResultParams;
use crate::services::work_runs::service::WorkRunsService;

pub struct SubmitResultParams {
    pub pr_url: String,
    pub exit_code: i32,
    pub tokens_used: i32,
    pub duration_ms: i32,
}

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

        if !self.notifier.take(&worker_id).await {
            return Ok(None);
        }

        self.work_runs_repo.find_oldest_pending_id(&self.db).await
    }

    pub async fn get_job(&self, id: Uuid) -> Result<WorkRun, WorkRunsError> {
        self.work_runs_repo.find_by_id(&self.db, id).await
    }

    pub async fn ack_job(&self, id: Uuid, worker_id: Uuid) -> Result<WorkRun, WorkRunsError> {
        self.work_runs_repo
            .acknowledge(&self.db, id, worker_id)
            .await
    }

    pub async fn submit_result(
        &self,
        id: Uuid,
        params: SubmitResultParams,
    ) -> Result<WorkRun, WorkRunsError> {
        let status = if params.exit_code == 0 {
            WorkRunStatus::Completed
        } else {
            WorkRunStatus::Failed
        };

        self.work_runs_repo
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
            .await
    }
}
