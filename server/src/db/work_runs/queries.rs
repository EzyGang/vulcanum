mod blocked;
mod insert;
mod limits;
pub(crate) mod prs;
mod reset;
mod result;
mod review_tickets;

use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRun, WorkRunListItem, WorkRunStatus, WorkRunType};
use vulcanum_shared::api::wire::JobRepo;

pub struct InsertWorkRunParams {
    pub team_id: Uuid,
    pub external_task_ref: String,
    pub task_title: Option<String>,
    pub task_slug: Option<String>,
    pub project_config_id: Uuid,
    pub repo_full_names: Vec<String>,
    pub status: WorkRunStatus,
    pub work_type: WorkRunType,
    pub parent_work_run_id: Option<Uuid>,
    pub review_target_pr_url: Option<String>,
    pub review_target_repo_full_name: Option<String>,
    pub github_installation_id: Option<i64>,
    pub github_delivery_id: Option<String>,
}

#[derive(Debug, Default)]
pub struct ReviewSiblingSummary {
    pub active_count: i64,
    pub failed_count: i64,
}

impl WorkRunsRepository {
    pub async fn find_dispatched_for_worker<'c, Q>(
        &self,
        db: Q,
        worker_id: Uuid,
    ) -> Result<Option<Uuid>, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        let row = sqlx::query!(
            r#"SELECT id FROM work_runs
             WHERE worker_id = $1 AND status = 'dispatched'::work_run_status
             ORDER BY updated_at ASC
             LIMIT 1"#,
            worker_id,
        )
        .fetch_optional(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(row.map(|row| row.id))
    }

    pub async fn list_repos<'c, Q>(
        &self,
        db: Q,
        work_run_id: Uuid,
    ) -> Result<Vec<JobRepo>, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        let rows = sqlx::query!(
            r#"SELECT repo_full_name, repo_url FROM work_run_repos
             WHERE work_run_id = $1 ORDER BY position ASC"#,
            work_run_id,
        )
        .fetch_all(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(rows
            .into_iter()
            .map(|row| JobRepo {
                full_name: row.repo_full_name,
                url: row.repo_url,
            })
            .collect())
    }

    pub async fn review_sibling_summary<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        parent_work_run_id: Uuid,
        current_work_run_id: Uuid,
    ) -> Result<ReviewSiblingSummary, WorkRunsError> {
        let row = sqlx::query!(
            r#"SELECT
             COUNT(*) FILTER (
                 WHERE status IN (
                     'pending'::work_run_status,
                     'dispatched'::work_run_status,
                     'running'::work_run_status
                 )
             ) AS "active_count!",
             COUNT(*) FILTER (
                 WHERE status = 'failed'::work_run_status
             ) AS "failed_count!"
             FROM work_runs
             WHERE parent_work_run_id = $1
             AND id != $2
             AND work_type = 'pull_request_review'::work_run_type"#,
            parent_work_run_id,
            current_work_run_id,
        )
        .fetch_one(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(ReviewSiblingSummary {
            active_count: row.active_count,
            failed_count: row.failed_count,
        })
    }

    pub async fn find_by_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<WorkRun, WorkRunsError> {
        sqlx::query_as!(
            WorkRun,
            r#"SELECT id, team_id, external_task_ref, task_title as "task_title?: String", task_slug as "task_slug?: String", project_config_id, worker_id, status as "status: WorkRunStatus",
             work_type as "work_type: WorkRunType", parent_work_run_id,
             review_target_pr_url, review_target_repo_full_name,
             github_installation_id, github_delivery_id,
             result_pr_url, result_exit_code, tokens_used, duration_ms,
             input_tokens as "input_tokens?: i64", output_tokens as "output_tokens?: i64",
             cache_read_tokens as "cache_read_tokens?: i64", cache_write_tokens as "cache_write_tokens?: i64",
             model_used,
             finish_status, result_summary, finish_blocked_reason, finish_next_column,
             created_at as "created_at!: chrono::DateTime<chrono::Utc>", updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
             FROM work_runs WHERE id = $1"#,
            id,
        )
        .fetch_optional(db)
        .await
        .map_err(WorkRunsError::from)?
        .ok_or(WorkRunsError::NotFound)
    }

    pub async fn list_all<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        team_id: Uuid,
        status: Option<WorkRunStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WorkRunListItem>, WorkRunsError> {
        sqlx::query_as!(
            WorkRunListItem,
            r#"SELECT wr.id, wr.team_id, wr.external_task_ref, wr.task_title as "task_title?: String",
             wr.task_slug as "task_slug?: String", wr.project_config_id, wr.worker_id,
             w.name as "worker_name: Option<String>",
             wr.status as "status: WorkRunStatus", wr.work_type as "work_type: WorkRunType", wr.parent_work_run_id,
             wr.review_target_pr_url, wr.review_target_repo_full_name,
             wr.github_installation_id, wr.github_delivery_id,
             wr.result_pr_url, wr.result_exit_code, wr.tokens_used, wr.duration_ms,
             wr.input_tokens as "input_tokens?: i64", wr.output_tokens as "output_tokens?: i64",
             wr.cache_read_tokens as "cache_read_tokens?: i64", wr.cache_write_tokens as "cache_write_tokens?: i64",
             wr.model_used,
             wr.finish_status, wr.result_summary, wr.finish_blocked_reason, wr.finish_next_column,
             wr.created_at as "created_at!: chrono::DateTime<chrono::Utc>"
             FROM work_runs wr LEFT JOIN workers w ON wr.worker_id = w.id
              WHERE wr.team_id = $1 AND ($2::work_run_status IS NULL OR wr.status = $2)
              ORDER BY wr.created_at DESC LIMIT $3 OFFSET $4"#,
            team_id,
            status as Option<WorkRunStatus>,
            limit,
            offset,
        )
        .fetch_all(db)
        .await
        .map_err(WorkRunsError::from)
    }

    pub async fn delete<'c, Q: Queryer<'c>>(&self, db: Q, id: Uuid) -> Result<(), WorkRunsError> {
        let rows = sqlx::query!("DELETE FROM work_runs WHERE id = $1", id)
            .execute(db)
            .await
            .map_err(WorkRunsError::from)?
            .rows_affected();

        if rows == 0 {
            return Err(WorkRunsError::NotFound);
        }

        Ok(())
    }
}

pub struct SetResultParams<'a> {
    pub pr_url: &'a str,
    pub exit_code: i32,
    pub tokens_used: i64,
    pub duration_ms: i64,
    pub status: WorkRunStatus,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub model_used: Option<&'a str>,
    pub finish_status: Option<&'a str>,
    pub result_summary: Option<&'a str>,
    pub finish_blocked_reason: Option<&'a str>,
    pub finish_next_column: Option<&'a str>,
}
