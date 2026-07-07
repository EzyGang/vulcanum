use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::models::task_board::model::TaskBoardTaskAugmentation;
use crate::models::work_runs::errors::WorkRunsError;

#[derive(Clone)]
pub struct TaskAugmentationsRepository {}

pub struct IncrementTaskUsageParams<'a> {
    pub team_id: Uuid,
    pub project_config_id: Uuid,
    pub external_task_ref: &'a str,
    pub tokens_used: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
}

impl Default for TaskAugmentationsRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskAugmentationsRepository {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    pub async fn increment_usage<'c, Q>(
        &self,
        db: Q,
        params: IncrementTaskUsageParams<'_>,
    ) -> Result<(), WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query(
            r#"INSERT INTO task_augmentations (
                team_id, project_config_id, external_task_ref, tokens_used, input_tokens,
                output_tokens, cache_read_tokens, cache_write_tokens, finished_runs_count
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1)
            ON CONFLICT (team_id, project_config_id, external_task_ref) DO UPDATE SET
                tokens_used = task_augmentations.tokens_used + EXCLUDED.tokens_used,
                input_tokens = task_augmentations.input_tokens + EXCLUDED.input_tokens,
                output_tokens = task_augmentations.output_tokens + EXCLUDED.output_tokens,
                cache_read_tokens = task_augmentations.cache_read_tokens + EXCLUDED.cache_read_tokens,
                cache_write_tokens = task_augmentations.cache_write_tokens + EXCLUDED.cache_write_tokens,
                finished_runs_count = task_augmentations.finished_runs_count + 1,
                updated_at = NOW()"#,
        )
        .bind(params.team_id)
        .bind(params.project_config_id)
        .bind(params.external_task_ref)
        .bind(params.tokens_used)
        .bind(params.input_tokens)
        .bind(params.output_tokens)
        .bind(params.cache_read_tokens)
        .bind(params.cache_write_tokens)
        .execute(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(())
    }

    pub async fn list_for_task_refs<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        project_config_id: Uuid,
        external_task_refs: &[String],
    ) -> Result<Vec<TaskBoardTaskAugmentation>, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        if external_task_refs.is_empty() {
            return Ok(Vec::new());
        }

        sqlx::query_as::<_, TaskBoardTaskAugmentation>(
            r#"WITH requested AS (
                SELECT task_ref, position
                FROM UNNEST($3::TEXT[]) WITH ORDINALITY AS refs(task_ref, position)
            )
            SELECT requested.task_ref AS external_task_ref,
                   augmentation.tokens_used,
                   augmentation.input_tokens,
                   augmentation.output_tokens,
                   augmentation.cache_read_tokens,
                   augmentation.cache_write_tokens,
                   augmentation.finished_runs_count,
                   augmentation.updated_at
            FROM requested
            JOIN task_augmentations augmentation
              ON augmentation.team_id = $1
             AND augmentation.project_config_id = $2
             AND augmentation.external_task_ref = requested.task_ref
            ORDER BY requested.position ASC"#,
        )
        .bind(team_id)
        .bind(project_config_id)
        .bind(external_task_refs)
        .fetch_all(db)
        .await
        .map_err(WorkRunsError::from)
    }
}

#[cfg(test)]
mod task_augmentations_tests;
