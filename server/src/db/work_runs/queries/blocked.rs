use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRunListItem, WorkRunStatus, WorkRunType};

impl WorkRunsRepository {
    pub async fn find_blocked_by_project<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        project_config_id: Uuid,
    ) -> Result<Vec<WorkRunListItem>, WorkRunsError> {
        sqlx::query_as!(
            WorkRunListItem,
            r#"SELECT wr.id, wr.team_id, wr.external_task_ref, NULL::TEXT as "task_title?: String",
             wr.external_task_ref as "task_slug!", wr.project_config_id, wr.worker_id,
             w.name as "worker_name: Option<String>",
             wr.status as "status: WorkRunStatus", wr.work_type as "work_type: WorkRunType", wr.parent_work_run_id,
             wr.review_target_pr_url, wr.review_target_repo_full_name,
             wr.result_pr_url, wr.result_exit_code, wr.tokens_used, wr.duration_ms,
             wr.input_tokens as "input_tokens?: i64", wr.output_tokens as "output_tokens?: i64",
             wr.cache_read_tokens as "cache_read_tokens?: i64", wr.cache_write_tokens as "cache_write_tokens?: i64",
             wr.model_used,
             wr.finish_status, wr.result_summary, wr.finish_blocked_reason, wr.finish_next_column,
             wr.created_at as "created_at!: chrono::DateTime<chrono::Utc>"
             FROM work_runs wr LEFT JOIN workers w ON wr.worker_id = w.id
             WHERE wr.project_config_id = $1 AND wr.status = 'failed'::work_run_status AND wr.finish_blocked_reason IS NOT NULL
             ORDER BY wr.created_at DESC"#,
            project_config_id,
        )
        .fetch_all(db)
        .await
        .map_err(WorkRunsError::from)
    }

    pub async fn reset_blocked_to_pending<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<(), WorkRunsError> {
        sqlx::query!(
            r#"UPDATE work_runs SET status = 'pending'::work_run_status, finish_blocked_reason = NULL, worker_id = NULL
             WHERE id = $1 AND status = 'failed'::work_run_status AND finish_blocked_reason IS NOT NULL"#,
            id,
        )
        .execute(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(())
    }
}
