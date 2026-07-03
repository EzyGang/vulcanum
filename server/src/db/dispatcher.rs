use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::models::dispatcher::errors::DispatchError;
use crate::models::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::models::workers::model::{Worker, WorkerStatus};

fn map_sqlx_error(err: sqlx::Error) -> DispatchError {
    DispatchError::Database(err)
}

#[derive(Clone, Default)]
pub struct DispatchRepository;

impl DispatchRepository {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub async fn find_available_workers<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<Vec<Worker>, DispatchError> {
        sqlx::query_as!(
            Worker,
            r#"SELECT id, team_id, name, refresh_token_hash, refresh_expires_at, status as "status: WorkerStatus", current_jobs, max_concurrent_jobs, last_seen_at, created_at as "created_at!: chrono::DateTime<chrono::Utc>"
             FROM workers WHERE status = 'idle'::worker_status AND current_jobs < max_concurrent_jobs ORDER BY last_seen_at ASC NULLS FIRST"#,
        )
        .fetch_all(db)
        .await
        .map_err(map_sqlx_error)
    }

    pub async fn find_pending_unassigned<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<Vec<WorkRun>, DispatchError> {
        sqlx::query_as!(
            WorkRun,
            r#"SELECT id, team_id, external_task_ref, project_config_id, worker_id, status as "status: WorkRunStatus",
             work_type as "work_type: WorkRunType", parent_work_run_id,
             review_target_pr_url, review_target_repo_full_name,
             result_pr_url, result_exit_code, tokens_used, duration_ms,
             input_tokens as "input_tokens?: i64", output_tokens as "output_tokens?: i64",
             cache_read_tokens as "cache_read_tokens?: i64", cache_write_tokens as "cache_write_tokens?: i64",
             model_used,
             finish_status, result_summary, finish_blocked_reason, finish_next_column,
             created_at as "created_at!: chrono::DateTime<chrono::Utc>", updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
             FROM work_runs WHERE status = 'pending'::work_run_status AND worker_id IS NULL AND finish_blocked_reason IS NULL
             ORDER BY created_at ASC"#,
        )
        .fetch_all(db)
        .await
        .map_err(map_sqlx_error)
    }

    pub async fn dispatch_to_worker<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        work_run_id: Uuid,
        worker_id: Uuid,
    ) -> Result<Option<WorkRun>, DispatchError> {
        sqlx::query_as!(
            WorkRun,
            r#"UPDATE work_runs SET worker_id = $2, status = 'dispatched'::work_run_status
             WHERE id = $1 AND status = 'pending'::work_run_status
             AND team_id = (SELECT team_id FROM workers WHERE id = $2)
             RETURNING id, team_id, external_task_ref, project_config_id, worker_id, status as "status: WorkRunStatus",
              work_type as "work_type: WorkRunType", parent_work_run_id,
              review_target_pr_url, review_target_repo_full_name,
             result_pr_url, result_exit_code, tokens_used, duration_ms,
             input_tokens as "input_tokens?: i64", output_tokens as "output_tokens?: i64",
             cache_read_tokens as "cache_read_tokens?: i64", cache_write_tokens as "cache_write_tokens?: i64",
             model_used,
             finish_status, result_summary, finish_blocked_reason, finish_next_column,
             created_at as "created_at!: chrono::DateTime<chrono::Utc>", updated_at as "updated_at!: chrono::DateTime<chrono::Utc>""#,
            work_run_id,
            worker_id,
        )
        .fetch_optional(db)
        .await
        .map_err(map_sqlx_error)
    }

    pub async fn increment_worker_jobs<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        worker_id: Uuid,
    ) -> Result<(), DispatchError> {
        sqlx::query!(
            "UPDATE workers SET current_jobs = current_jobs + 1 WHERE id = $1",
            worker_id
        )
        .execute(db)
        .await
        .map(|_| ())
        .map_err(map_sqlx_error)
    }
}
