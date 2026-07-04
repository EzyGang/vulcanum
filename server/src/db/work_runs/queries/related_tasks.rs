use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{TaskBoardRelatedWorkRunRow, WorkRunStatus, WorkRunType};

impl WorkRunsRepository {
    pub async fn list_latest_related_for_task_refs<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        project_config_id: Uuid,
        external_task_refs: &[String],
        limit_per_task: i64,
    ) -> Result<Vec<TaskBoardRelatedWorkRunRow>, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        if external_task_refs.is_empty() || limit_per_task <= 0 {
            return Ok(Vec::new());
        }

        sqlx::query_as!(
            TaskBoardRelatedWorkRunRow,
            r#"WITH requested AS (
                SELECT task_ref, position
                FROM UNNEST($3::TEXT[]) WITH ORDINALITY AS refs(task_ref, position)
            )
            SELECT requested.task_ref as "external_task_ref!",
                   wr.id,
                   wr.status as "status: WorkRunStatus",
                   wr.work_type as "work_type: WorkRunType",
                   wr.tokens_used,
                   wr.input_tokens as "input_tokens?: i64",
                   wr.output_tokens as "output_tokens?: i64",
                   wr.cache_read_tokens as "cache_read_tokens?: i64",
                   wr.cache_write_tokens as "cache_write_tokens?: i64",
                   wr.model_used,
                   wr.created_at as "created_at!: chrono::DateTime<chrono::Utc>"
            FROM requested
            JOIN LATERAL (
                SELECT id, status, work_type, tokens_used, input_tokens, output_tokens,
                       cache_read_tokens, cache_write_tokens, model_used, created_at
                FROM work_runs
                WHERE team_id = $1
                  AND project_config_id = $2
                  AND external_task_ref = requested.task_ref
                ORDER BY created_at DESC, id DESC
                LIMIT $4
            ) wr ON TRUE
            ORDER BY requested.position ASC, wr.created_at DESC, wr.id DESC"#,
            team_id,
            project_config_id,
            external_task_refs,
            limit_per_task,
        )
        .fetch_all(db)
        .await
        .map_err(WorkRunsError::from)
    }
}
