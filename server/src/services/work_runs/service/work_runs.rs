use uuid::Uuid;

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
            tracing::warn!("failed to update last_seen for worker {worker_id}: {e}");
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

        let kaneo_project_id;
        let kaneo_workspace_id;

        match config {
            Ok(c) => {
                kaneo_project_id = c.kaneo_project_id;
                kaneo_workspace_id = c.kaneo_workspace_id;
            }
            Err(_) => {
                tracing::warn!(
                    "Project config {} not found for work_run {}",
                    run.project_config_id,
                    id
                );
                kaneo_project_id = String::new();
                kaneo_workspace_id = String::new();
            }
        }

        Ok(JobResponse {
            prompt_text: run.prompt_text,
            repo_url: run.repo_url,
            agents_md: run.agents_md,
            external_task_ref: run.external_task_ref,
            kaneo_instance: self.kaneo.instance.clone(),
            kaneo_api_key: self.kaneo.api_key.clone(),
            kaneo_project_id,
            kaneo_workspace_id,
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
