use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::models::task_board::model::{TaskBoardProjectUsage, TaskBoardUsageCounters};
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRunStatus, WorkRunType};

#[derive(Clone, Default)]
pub struct ProjectUsageRepository {}

pub struct IncrementProjectUsageParams {
    pub project_config_id: Uuid,
    pub tokens_used: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub work_type: WorkRunType,
    pub status: WorkRunStatus,
}

impl ProjectUsageRepository {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    pub async fn increment_daily<'c, Q>(
        &self,
        db: Q,
        params: IncrementProjectUsageParams,
    ) -> Result<(), WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query!(
            r#"INSERT INTO project_usage_daily (
                project_config_id, usage_date, tokens_used, input_tokens, output_tokens,
                cache_read_tokens, cache_write_tokens, finished_runs_count,
                implementation_runs_count, review_runs_count, successful_runs_count,
                failed_runs_count
            )
            VALUES (
                $1, (statement_timestamp() AT TIME ZONE 'UTC')::DATE, $2, $3, $4, $5, $6, 1,
                CASE WHEN $7 = 'implementation'::work_run_type THEN 1 ELSE 0 END,
                CASE WHEN $7 = 'pull_request_review'::work_run_type THEN 1 ELSE 0 END,
                CASE WHEN $8 = 'completed'::work_run_status THEN 1 ELSE 0 END,
                CASE WHEN $8 = 'failed'::work_run_status THEN 1 ELSE 0 END
            )
            ON CONFLICT (project_config_id, usage_date) DO UPDATE SET
                tokens_used = project_usage_daily.tokens_used + EXCLUDED.tokens_used,
                input_tokens = project_usage_daily.input_tokens + EXCLUDED.input_tokens,
                output_tokens = project_usage_daily.output_tokens + EXCLUDED.output_tokens,
                cache_read_tokens = project_usage_daily.cache_read_tokens + EXCLUDED.cache_read_tokens,
                cache_write_tokens = project_usage_daily.cache_write_tokens + EXCLUDED.cache_write_tokens,
                finished_runs_count = project_usage_daily.finished_runs_count + 1,
                implementation_runs_count = project_usage_daily.implementation_runs_count
                    + EXCLUDED.implementation_runs_count,
                review_runs_count = project_usage_daily.review_runs_count
                    + EXCLUDED.review_runs_count,
                successful_runs_count = project_usage_daily.successful_runs_count
                    + EXCLUDED.successful_runs_count,
                failed_runs_count = project_usage_daily.failed_runs_count
                    + EXCLUDED.failed_runs_count"#,
            params.project_config_id,
            params.tokens_used,
            params.input_tokens,
            params.output_tokens,
            params.cache_read_tokens,
            params.cache_write_tokens,
            &params.work_type as &WorkRunType,
            &params.status as &WorkRunStatus,
        )
        .execute(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(())
    }

    pub async fn summary<'c, Q>(
        &self,
        db: Q,
        project_config_id: Uuid,
    ) -> Result<TaskBoardProjectUsage, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        let row = sqlx::query_as!(
            ProjectUsageAggregateRow,
            r#"WITH bounds AS (
                SELECT
                    (statement_timestamp() AT TIME ZONE 'UTC')::DATE AS today,
                    DATE_TRUNC('week', statement_timestamp() AT TIME ZONE 'UTC')::DATE AS week_start
            )
            SELECT
                COALESCE(SUM(tokens_used), 0)::BIGINT AS "total_tokens_used!",
                COALESCE(SUM(input_tokens), 0)::BIGINT AS "total_input_tokens!",
                COALESCE(SUM(output_tokens), 0)::BIGINT AS "total_output_tokens!",
                COALESCE(SUM(cache_read_tokens), 0)::BIGINT AS "total_cache_read_tokens!",
                COALESCE(SUM(cache_write_tokens), 0)::BIGINT AS "total_cache_write_tokens!",
                COALESCE(SUM(finished_runs_count), 0)::BIGINT AS "total_finished_runs_count!",
                COALESCE(SUM(implementation_runs_count), 0)::BIGINT
                    AS "total_implementation_runs_count!",
                COALESCE(SUM(review_runs_count), 0)::BIGINT AS "total_review_runs_count!",
                COALESCE(SUM(successful_runs_count), 0)::BIGINT AS "total_successful_runs_count!",
                COALESCE(SUM(failed_runs_count), 0)::BIGINT AS "total_failed_runs_count!",
                COALESCE(SUM(tokens_used) FILTER (
                    WHERE usage_date BETWEEN bounds.week_start AND bounds.today
                ), 0)::BIGINT AS "week_tokens_used!",
                COALESCE(SUM(input_tokens) FILTER (
                    WHERE usage_date BETWEEN bounds.week_start AND bounds.today
                ), 0)::BIGINT AS "week_input_tokens!",
                COALESCE(SUM(output_tokens) FILTER (
                    WHERE usage_date BETWEEN bounds.week_start AND bounds.today
                ), 0)::BIGINT AS "week_output_tokens!",
                COALESCE(SUM(cache_read_tokens) FILTER (
                    WHERE usage_date BETWEEN bounds.week_start AND bounds.today
                ), 0)::BIGINT AS "week_cache_read_tokens!",
                COALESCE(SUM(cache_write_tokens) FILTER (
                    WHERE usage_date BETWEEN bounds.week_start AND bounds.today
                ), 0)::BIGINT AS "week_cache_write_tokens!",
                COALESCE(SUM(finished_runs_count) FILTER (
                    WHERE usage_date BETWEEN bounds.week_start AND bounds.today
                ), 0)::BIGINT AS "week_finished_runs_count!",
                COALESCE(SUM(implementation_runs_count) FILTER (
                    WHERE usage_date BETWEEN bounds.week_start AND bounds.today
                ), 0)::BIGINT AS "week_implementation_runs_count!",
                COALESCE(SUM(review_runs_count) FILTER (
                    WHERE usage_date BETWEEN bounds.week_start AND bounds.today
                ), 0)::BIGINT AS "week_review_runs_count!",
                COALESCE(SUM(successful_runs_count) FILTER (
                    WHERE usage_date BETWEEN bounds.week_start AND bounds.today
                ), 0)::BIGINT AS "week_successful_runs_count!",
                COALESCE(SUM(failed_runs_count) FILTER (
                    WHERE usage_date BETWEEN bounds.week_start AND bounds.today
                ), 0)::BIGINT AS "week_failed_runs_count!"
            FROM project_usage_daily
            CROSS JOIN bounds
            WHERE project_config_id = $1"#,
            project_config_id,
        )
        .fetch_one(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(row.into())
    }
}

#[derive(Debug)]
struct ProjectUsageAggregateRow {
    total_tokens_used: i64,
    total_input_tokens: i64,
    total_output_tokens: i64,
    total_cache_read_tokens: i64,
    total_cache_write_tokens: i64,
    total_finished_runs_count: i64,
    total_implementation_runs_count: i64,
    total_review_runs_count: i64,
    total_successful_runs_count: i64,
    total_failed_runs_count: i64,
    week_tokens_used: i64,
    week_input_tokens: i64,
    week_output_tokens: i64,
    week_cache_read_tokens: i64,
    week_cache_write_tokens: i64,
    week_finished_runs_count: i64,
    week_implementation_runs_count: i64,
    week_review_runs_count: i64,
    week_successful_runs_count: i64,
    week_failed_runs_count: i64,
}

impl From<ProjectUsageAggregateRow> for TaskBoardProjectUsage {
    fn from(row: ProjectUsageAggregateRow) -> Self {
        Self {
            total: TaskBoardUsageCounters {
                tokens_used: row.total_tokens_used,
                input_tokens: row.total_input_tokens,
                output_tokens: row.total_output_tokens,
                cache_read_tokens: row.total_cache_read_tokens,
                cache_write_tokens: row.total_cache_write_tokens,
                finished_runs_count: row.total_finished_runs_count,
                implementation_runs_count: row.total_implementation_runs_count,
                review_runs_count: row.total_review_runs_count,
                successful_runs_count: row.total_successful_runs_count,
                failed_runs_count: row.total_failed_runs_count,
            },
            this_week: TaskBoardUsageCounters {
                tokens_used: row.week_tokens_used,
                input_tokens: row.week_input_tokens,
                output_tokens: row.week_output_tokens,
                cache_read_tokens: row.week_cache_read_tokens,
                cache_write_tokens: row.week_cache_write_tokens,
                finished_runs_count: row.week_finished_runs_count,
                implementation_runs_count: row.week_implementation_runs_count,
                review_runs_count: row.week_review_runs_count,
                successful_runs_count: row.week_successful_runs_count,
                failed_runs_count: row.week_failed_runs_count,
            },
        }
    }
}

#[cfg(test)]
mod project_usage_tests;
