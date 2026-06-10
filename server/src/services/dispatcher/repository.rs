use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::dispatcher::errors::DispatchError;
use crate::services::work_runs::model::{WorkRun, WorkRunStatus};
use crate::services::workers::model::{Worker, WorkerStatus};

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
            r#"SELECT id, team_id, name, refresh_token_hash, refresh_expires_at, last_seen,
             status as "status: WorkerStatus", capabilities, created_at as "created_at!: chrono::DateTime<chrono::Utc>",
             active_jobs, max_concurrent_jobs, consecutive_errors
             FROM workers WHERE active_jobs < max_concurrent_jobs AND status IN ('idle'::worker_status, 'busy'::worker_status)
             ORDER BY last_seen DESC NULLS LAST"#,
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
             prompt_text, repo_url, agents_md, task_title, task_slug,
             result_pr_url, result_exit_code, tokens_used, duration_ms,
             input_tokens as "input_tokens?: i64", output_tokens as "output_tokens?: i64",
             cache_read_tokens as "cache_read_tokens?: i64", cache_write_tokens as "cache_write_tokens?: i64",
             model_used,
             finish_status, finish_summary, finish_blocked_reason, finish_next_column,
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
             prompt_text, repo_url, agents_md, task_title, task_slug,
             result_pr_url, result_exit_code, tokens_used, duration_ms,
             input_tokens as "input_tokens?: i64", output_tokens as "output_tokens?: i64",
             cache_read_tokens as "cache_read_tokens?: i64", cache_write_tokens as "cache_write_tokens?: i64",
             model_used,
             finish_status, finish_summary, finish_blocked_reason, finish_next_column,
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
            "UPDATE workers SET active_jobs = active_jobs + 1, status = 'busy'::worker_status
             WHERE id = $1 AND active_jobs < max_concurrent_jobs",
            worker_id,
        )
        .execute(db)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }
}
