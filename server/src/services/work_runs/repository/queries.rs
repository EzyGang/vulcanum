use sqlx::PgConnection;
use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::{WorkRun, WorkRunListItem, WorkRunStatus};
use crate::services::work_runs::repository::WorkRunsRepository;
use vulcanum_shared::api_types::JobRepo;

pub struct InsertWorkRunParams {
    pub team_id: Uuid,
    pub external_task_ref: String,
    pub project_config_id: Uuid,
    pub prompt_text: String,
    pub repo_url: String,
    pub repo_full_names: Vec<String>,
    pub agents_md: String,
    pub status: WorkRunStatus,
    pub task_title: Option<String>,
    pub task_slug: Option<String>,
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
            r#"INSERT INTO work_runs (id, team_id, external_task_ref, project_config_id, status, prompt_text, repo_url, agents_md, task_title, task_slug)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING id, team_id, external_task_ref, project_config_id, worker_id, status as "status: WorkRunStatus", prompt_text,
                        repo_url, agents_md, task_title, task_slug,
                        result_pr_url, result_exit_code, tokens_used, duration_ms,
                        input_tokens as "input_tokens?: i64", output_tokens as "output_tokens?: i64",
                        cache_read_tokens as "cache_read_tokens?: i64", cache_write_tokens as "cache_write_tokens?: i64",
                        model_used,
                        finish_status, finish_summary, finish_blocked_reason, finish_next_column,
                        created_at as "created_at!: chrono::DateTime<chrono::Utc>", updated_at as "updated_at!: chrono::DateTime<chrono::Utc>""#,
            id,
            params.team_id,
            &params.external_task_ref,
            params.project_config_id,
            &params.status as &WorkRunStatus,
            &params.prompt_text,
            &params.repo_url,
            &params.agents_md,
            params.task_title.as_deref(),
            params.task_slug.as_deref(),
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
            r#"INSERT INTO work_runs (id, team_id, external_task_ref, project_config_id, status, prompt_text, repo_url, agents_md, task_title, task_slug)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
              ON CONFLICT DO NOTHING"#,
            id,
            params.team_id,
            &params.external_task_ref,
            params.project_config_id,
            &params.status as &WorkRunStatus,
            &params.prompt_text,
            &params.repo_url,
            &params.agents_md,
            params.task_title.as_deref(),
            params.task_slug.as_deref(),
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
        for (position, full_name) in repo_full_names.iter().enumerate() {
            sqlx::query!(
                r#"INSERT INTO work_run_repos (work_run_id, repo_full_name, repo_url, position)
                 VALUES ($1, $2, $3, $4)"#,
                work_run_id,
                full_name,
                format!("https://github.com/{full_name}"),
                position as i32,
            )
            .execute(db)
            .await
            .map_err(WorkRunsError::from)?;
        }

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

    pub async fn find_by_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<WorkRun, WorkRunsError> {
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
             FROM work_runs WHERE id = $1"#,
            id,
        )
        .fetch_optional(db)
        .await
        .map_err(WorkRunsError::from)?
        .ok_or(WorkRunsError::NotFound)
    }

    pub async fn find_oldest_pending_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<Option<(Uuid, String)>, WorkRunsError> {
        let row = sqlx::query!(
            r#"SELECT id, external_task_ref FROM work_runs WHERE status = 'pending'::work_run_status ORDER BY created_at ASC LIMIT 1"#,
        )
        .fetch_optional(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(row.map(|r| (r.id, r.external_task_ref)))
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
             wr.status as "status: WorkRunStatus", wr.prompt_text, wr.repo_url,
             wr.task_title, wr.task_slug,
             wr.result_pr_url, wr.result_exit_code, wr.tokens_used, wr.duration_ms,
             wr.input_tokens as "input_tokens?: i64", wr.output_tokens as "output_tokens?: i64",
             wr.cache_read_tokens as "cache_read_tokens?: i64", wr.cache_write_tokens as "cache_write_tokens?: i64",
             wr.model_used,
             wr.finish_status, wr.finish_summary, wr.finish_blocked_reason, wr.finish_next_column,
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

    pub async fn reset_orphaned_dispatched<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        threshold_secs: i64,
    ) -> Result<u64, WorkRunsError> {
        let rows = sqlx::query!(
            r#"UPDATE work_runs SET status = 'pending'::work_run_status, worker_id = NULL
             WHERE status = 'dispatched'::work_run_status
             AND updated_at < NOW() - INTERVAL '1 second' * $1
             AND finish_blocked_reason IS NULL"#,
            threshold_secs as f64,
        )
        .execute(db)
        .await
        .map_err(WorkRunsError::from)?
        .rows_affected();

        Ok(rows)
    }

    pub async fn reset_orphaned_worker_runs<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<u64, WorkRunsError> {
        let rows = sqlx::query!(
            r#"UPDATE work_runs SET status = 'pending'::work_run_status, worker_id = NULL
             WHERE status IN ('dispatched'::work_run_status, 'running'::work_run_status)
             AND worker_id IS NULL
             AND finish_blocked_reason IS NULL"#,
        )
        .execute(db)
        .await
        .map_err(WorkRunsError::from)?
        .rows_affected();

        Ok(rows)
    }

    pub async fn reset_stalled_running<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        threshold_secs: i64,
    ) -> Result<u64, WorkRunsError> {
        let rows = sqlx::query!(
            r#"WITH reset_runs AS (
                UPDATE work_runs SET status = 'pending'::work_run_status, worker_id = NULL
                WHERE status = 'running'::work_run_status
                AND updated_at < NOW() - INTERVAL '1 second' * $1
                AND finish_blocked_reason IS NULL
                RETURNING worker_id
            )
            SELECT COUNT(DISTINCT worker_id) AS affected_workers
            FROM reset_runs WHERE worker_id IS NOT NULL"#,
            threshold_secs as f64,
        )
        .fetch_one(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(rows.affected_workers.unwrap_or(0) as u64)
    }

    pub async fn reset_worker_active_jobs<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        worker_id: Uuid,
    ) -> Result<u64, WorkRunsError> {
        self.reset_worker_active_jobs_raw(db, worker_id)
            .await
            .map_err(WorkRunsError::Database)
    }

    pub async fn reset_worker_active_jobs_raw<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        worker_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let rows = sqlx::query!(
            r#"UPDATE work_runs SET status = 'pending'::work_run_status, worker_id = NULL
             WHERE worker_id = $1
             AND status IN ('dispatched'::work_run_status, 'running'::work_run_status)"#,
            worker_id,
        )
        .execute(db)
        .await?
        .rows_affected();

        Ok(rows)
    }

    pub async fn reset_worker_dispatched<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        worker_id: Uuid,
        threshold_secs: i64,
    ) -> Result<u64, WorkRunsError> {
        let rows = sqlx::query!(
            r#"UPDATE work_runs SET status = 'pending'::work_run_status, worker_id = NULL
             WHERE worker_id = $1 AND status = 'dispatched'::work_run_status
             AND updated_at < NOW() - INTERVAL '1 second' * $2"#,
            worker_id,
            threshold_secs as f64,
        )
        .execute(db)
        .await
        .map_err(WorkRunsError::from)?
        .rows_affected();

        Ok(rows)
    }

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
             prompt_text, repo_url, agents_md, task_title, task_slug,
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
             prompt_text, repo_url, agents_md, task_title, task_slug,
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
             finish_next_column = $15
             WHERE id = $1 AND status = 'running'::work_run_status
             RETURNING id, team_id, external_task_ref, project_config_id, worker_id, status as "status: WorkRunStatus",
             prompt_text, repo_url, agents_md, task_title, task_slug,
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
        )
        .fetch_optional(db)
        .await
        .map_err(WorkRunsError::from)?
        .ok_or(WorkRunsError::InvalidStatusTransition)
    }

    pub async fn replace_pr_urls(
        &self,
        db: &mut PgConnection,
        work_run_id: Uuid,
        pr_urls: &[String],
    ) -> Result<(), WorkRunsError> {
        sqlx::query!(
            "DELETE FROM work_run_prs WHERE work_run_id = $1",
            work_run_id
        )
        .execute(&mut *db)
        .await
        .map_err(WorkRunsError::from)?;

        for (position, pr_url) in pr_urls.iter().enumerate() {
            sqlx::query!(
                r#"INSERT INTO work_run_prs (work_run_id, pr_url, position)
                 VALUES ($1, $2, $3)"#,
                work_run_id,
                pr_url,
                position as i32,
            )
            .execute(&mut *db)
            .await
            .map_err(WorkRunsError::from)?;
        }

        Ok(())
    }

    pub async fn find_blocked_by_project<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        project_config_id: Uuid,
    ) -> Result<Vec<WorkRunListItem>, WorkRunsError> {
        sqlx::query_as!(
            WorkRunListItem,
            r#"SELECT wr.id, wr.team_id, wr.external_task_ref, wr.project_config_id, wr.worker_id,
             w.name as "worker_name: Option<String>",
             wr.status as "status: WorkRunStatus", wr.prompt_text, wr.repo_url,
             wr.task_title, wr.task_slug,
             wr.result_pr_url, wr.result_exit_code, wr.tokens_used, wr.duration_ms,
             wr.input_tokens as "input_tokens?: i64", wr.output_tokens as "output_tokens?: i64",
             wr.cache_read_tokens as "cache_read_tokens?: i64", wr.cache_write_tokens as "cache_write_tokens?: i64",
             wr.model_used,
             wr.finish_status, wr.finish_summary, wr.finish_blocked_reason, wr.finish_next_column,
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
    pub finish_summary: Option<&'a str>,
    pub finish_blocked_reason: Option<&'a str>,
    pub finish_next_column: Option<&'a str>,
}
