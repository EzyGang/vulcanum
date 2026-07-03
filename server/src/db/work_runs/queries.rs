mod blocked;
mod limits;
pub(crate) mod prs;
mod reset;
mod result;

use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRun, WorkRunListItem, WorkRunStatus, WorkRunType};
use crate::util::github::github_repo_url;
use vulcanum_shared::api_types::JobRepo;

pub struct InsertWorkRunParams {
    pub team_id: Uuid,
    pub external_task_ref: String,
    pub project_config_id: Uuid,
    pub repo_full_names: Vec<String>,
    pub status: WorkRunStatus,
    pub work_type: WorkRunType,
    pub parent_work_run_id: Option<Uuid>,
    pub review_target_pr_url: Option<String>,
    pub review_target_repo_full_name: Option<String>,
}

#[derive(Debug, Default)]
pub struct ReviewSiblingSummary {
    pub active_count: i64,
    pub failed_count: i64,
}

impl WorkRunsRepository {
    pub async fn insert_work_run<'c, Q>(
        &self,
        db: Q,
        params: InsertWorkRunParams,
    ) -> Result<WorkRun, WorkRunsError>
    where
        Q: Queryer<'c> + Copy,
    {
        let id = Uuid::new_v4();

        let run = sqlx::query_as!(
            WorkRun,
            r#"INSERT INTO work_runs (id, team_id, external_task_ref, project_config_id, status, work_type, parent_work_run_id,
             review_target_pr_url, review_target_repo_full_name)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
              RETURNING id, team_id, external_task_ref, project_config_id, worker_id, status as "status: WorkRunStatus",
                         work_type as "work_type: WorkRunType", parent_work_run_id,
                         review_target_pr_url, review_target_repo_full_name,
                         result_pr_url, result_exit_code, tokens_used, duration_ms,
                        input_tokens as "input_tokens?: i64", output_tokens as "output_tokens?: i64",
                        cache_read_tokens as "cache_read_tokens?: i64", cache_write_tokens as "cache_write_tokens?: i64",
                        model_used,
                        finish_status, result_summary, finish_blocked_reason, finish_next_column,
                        created_at as "created_at!: chrono::DateTime<chrono::Utc>", updated_at as "updated_at!: chrono::DateTime<chrono::Utc>""#,
            id,
            params.team_id,
            &params.external_task_ref,
            params.project_config_id,
            &params.status as &WorkRunStatus,
            &params.work_type as &WorkRunType,
            params.parent_work_run_id,
            params.review_target_pr_url.as_deref(),
            params.review_target_repo_full_name.as_deref(),
        )
        .fetch_one(db)
        .await
        .map_err(WorkRunsError::from)?;
        self.insert_repos(db, run.id, &params.repo_full_names)
            .await?;
        Ok(run)
    }

    pub async fn insert_work_run_if_not_active<'c, Q>(
        &self,
        db: Q,
        params: InsertWorkRunParams,
    ) -> Result<bool, WorkRunsError>
    where
        Q: Queryer<'c> + Copy,
    {
        let id = Uuid::new_v4();

        let result = sqlx::query!(
            r#"INSERT INTO work_runs (id, team_id, external_task_ref, project_config_id, status, work_type, parent_work_run_id,
             review_target_pr_url, review_target_repo_full_name)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               ON CONFLICT DO NOTHING"#,
            id,
            params.team_id,
            &params.external_task_ref,
            params.project_config_id,
            &params.status as &WorkRunStatus,
            &params.work_type as &WorkRunType,
            params.parent_work_run_id,
            params.review_target_pr_url.as_deref(),
            params.review_target_repo_full_name.as_deref(),
        )
        .execute(db)
        .await
        .map_err(WorkRunsError::from)?;

        if result.rows_affected() == 0 {
            return Ok(false);
        }

        self.insert_repos(db, id, &params.repo_full_names).await?;
        Ok(true)
    }

    async fn insert_repos<'c, Q>(
        &self,
        db: Q,
        work_run_id: Uuid,
        repo_full_names: &[String],
    ) -> Result<(), WorkRunsError>
    where
        Q: Queryer<'c> + Copy,
    {
        if repo_full_names.is_empty() {
            return Ok(());
        }

        let repo_urls = repo_full_names
            .iter()
            .map(|name| github_repo_url(name))
            .collect::<Vec<String>>();
        let positions = (0..repo_full_names.len() as i32).collect::<Vec<i32>>();

        sqlx::query(
            r#"INSERT INTO work_run_repos (work_run_id, repo_full_name, repo_url, position)
             SELECT $1, repo_full_name, repo_url, position
             FROM UNNEST($2::text[], $3::text[], $4::int4[]) AS repos(repo_full_name, repo_url, position)"#,
        )
        .bind(work_run_id)
        .bind(repo_full_names)
        .bind(&repo_urls)
        .bind(&positions)
        .execute(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(())
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
        let (active_count, failed_count): (i64, i64) = sqlx::query_as(
            r#"SELECT
             COUNT(*) FILTER (WHERE status IN ('pending'::work_run_status, 'dispatched'::work_run_status, 'running'::work_run_status)) AS active_count,
             COUNT(*) FILTER (WHERE status = 'failed'::work_run_status) AS failed_count
             FROM work_runs
             WHERE parent_work_run_id = $1
             AND id != $2
             AND work_type = 'pull_request_review'::work_run_type"#,
        )
        .bind(parent_work_run_id)
        .bind(current_work_run_id)
        .fetch_one(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(ReviewSiblingSummary {
            active_count,
            failed_count,
        })
    }

    pub async fn find_by_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<WorkRun, WorkRunsError> {
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
            r#"SELECT wr.id, wr.team_id, wr.external_task_ref, wr.project_config_id, wr.worker_id,
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
