use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::services::work_runs::repository::queries::SetResultParams;
use crate::services::work_runs::repository::WorkRunsRepository;

impl WorkRunsRepository {
    pub async fn acknowledge<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        worker_id: Uuid,
    ) -> Result<WorkRun, WorkRunsError> {
        sqlx::query_as!(
            WorkRun,
            r#"UPDATE work_runs SET status = 'running'::work_run_status
             WHERE id = $1 AND worker_id = $2 AND status = 'dispatched'::work_run_status
             RETURNING id, team_id, external_task_ref, project_config_id, worker_id, status as "status: WorkRunStatus",
              work_type as "work_type: WorkRunType", parent_work_run_id,
              prompt_text, repo_url, agents_md, task_body, task_title, task_slug,
              review_target_pr_url, review_target_repo_full_name, review_url, review_body, review_already_exists,
             result_pr_url, result_exit_code, tokens_used, duration_ms,
             input_tokens as "input_tokens?: i64", output_tokens as "output_tokens?: i64",
             cache_read_tokens as "cache_read_tokens?: i64", cache_write_tokens as "cache_write_tokens?: i64",
             model_used,
             finish_status, finish_summary, finish_blocked_reason, finish_next_column,
             created_at as "created_at!: chrono::DateTime<chrono::Utc>", updated_at as "updated_at!: chrono::DateTime<chrono::Utc>""#,
            id,
            worker_id,
        )
        .fetch_optional(db)
        .await
        .map_err(WorkRunsError::from)?
        .ok_or(WorkRunsError::AlreadyClaimed)
    }

    pub async fn force_fail<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<Option<WorkRun>, WorkRunsError> {
        sqlx::query_as!(
            WorkRun,
            r#"UPDATE work_runs SET status = 'failed'::work_run_status, result_exit_code = 1, tokens_used = 0, duration_ms = 0,
             input_tokens = 0, output_tokens = 0, cache_read_tokens = 0, cache_write_tokens = 0
             WHERE id = $1 AND status IN ('running'::work_run_status, 'dispatched'::work_run_status)
             RETURNING id, team_id, external_task_ref, project_config_id, worker_id, status as "status: WorkRunStatus",
              work_type as "work_type: WorkRunType", parent_work_run_id,
              prompt_text, repo_url, agents_md, task_body, task_title, task_slug,
              review_target_pr_url, review_target_repo_full_name, review_url, review_body, review_already_exists,
             result_pr_url, result_exit_code, tokens_used, duration_ms,
             input_tokens as "input_tokens?: i64", output_tokens as "output_tokens?: i64",
             cache_read_tokens as "cache_read_tokens?: i64", cache_write_tokens as "cache_write_tokens?: i64",
             model_used,
             finish_status, finish_summary, finish_blocked_reason, finish_next_column,
             created_at as "created_at!: chrono::DateTime<chrono::Utc>", updated_at as "updated_at!: chrono::DateTime<chrono::Utc>""#,
            id,
        )
        .fetch_optional(db)
        .await
        .map_err(WorkRunsError::from)
    }

    pub async fn set_result<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        params: SetResultParams<'_>,
    ) -> Result<WorkRun, WorkRunsError> {
        sqlx::query_as!(
            WorkRun,
            r#"UPDATE work_runs SET result_pr_url = $2, result_exit_code = $3, tokens_used = $4,
             duration_ms = $5, status = $6, input_tokens = $7, output_tokens = $8,
             cache_read_tokens = $9, cache_write_tokens = $10, model_used = $11,
             finish_status = $12, finish_summary = $13, finish_blocked_reason = $14,
             finish_next_column = $15, review_url = $16, review_body = $17,
             review_already_exists = $18
             WHERE id = $1 AND status = 'running'::work_run_status
             RETURNING id, team_id, external_task_ref, project_config_id, worker_id, status as "status: WorkRunStatus",
              work_type as "work_type: WorkRunType", parent_work_run_id,
              prompt_text, repo_url, agents_md, task_body, task_title, task_slug,
              review_target_pr_url, review_target_repo_full_name, review_url, review_body, review_already_exists,
             result_pr_url, result_exit_code, tokens_used, duration_ms,
             input_tokens as "input_tokens?: i64", output_tokens as "output_tokens?: i64",
             cache_read_tokens as "cache_read_tokens?: i64", cache_write_tokens as "cache_write_tokens?: i64",
             model_used,
             finish_status, finish_summary, finish_blocked_reason, finish_next_column,
             created_at as "created_at!: chrono::DateTime<chrono::Utc>", updated_at as "updated_at!: chrono::DateTime<chrono::Utc>""#,
            id,
            params.pr_url,
            params.exit_code,
            params.tokens_used,
            params.duration_ms,
            &params.status as &WorkRunStatus,
            params.input_tokens,
            params.output_tokens,
            params.cache_read_tokens,
            params.cache_write_tokens,
            params.model_used,
            params.finish_status,
            params.finish_summary,
            params.finish_blocked_reason,
            params.finish_next_column,
            params.review_url,
            params.review_body,
            params.review_already_exists,
        )
        .fetch_optional(db)
        .await
        .map_err(WorkRunsError::from)?
        .ok_or(WorkRunsError::InvalidStatusTransition)
    }

    pub async fn replace_pr_urls<'c, Q>(
        &self,
        db: Q,
        work_run_id: Uuid,
        pr_urls: &[String],
    ) -> Result<(), WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        if pr_urls.is_empty() {
            sqlx::query!(
                "DELETE FROM work_run_prs WHERE work_run_id = $1",
                work_run_id
            )
            .execute(db)
            .await
            .map_err(WorkRunsError::from)?;

            return Ok(());
        }

        let mut query = QueryBuilder::<Postgres>::new(
            "WITH deleted AS (DELETE FROM work_run_prs WHERE work_run_id = ",
        );
        query.push_bind(work_run_id);
        query.push(") INSERT INTO work_run_prs (work_run_id, pr_url, position) ");

        query.push_values(
            pr_urls.iter().enumerate(),
            |mut builder, (position, pr_url)| {
                builder
                    .push_bind(work_run_id)
                    .push_bind(pr_url)
                    .push_bind(position as i32);
            },
        );

        query
            .build()
            .execute(db)
            .await
            .map_err(WorkRunsError::from)?;

        Ok(())
    }
}
