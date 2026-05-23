use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::{WorkRun, WorkRunListItem, WorkRunStatus};
use crate::services::work_runs::repository::WorkRunsRepository;

pub struct InsertWorkRunParams {
    pub external_task_ref: String,
    pub project_config_id: Uuid,
    pub prompt_text: String,
    pub repo_url: String,
    pub agents_md: String,
    pub status: WorkRunStatus,
}

impl WorkRunsRepository {
    #[allow(dead_code)]
    pub async fn insert_work_run<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        params: InsertWorkRunParams,
    ) -> Result<WorkRun, WorkRunsError> {
        let id = Uuid::new_v4();

        sqlx::query_as!(
            WorkRun,
            r#"INSERT INTO work_runs (id, external_task_ref, project_config_id, status, prompt_text, repo_url, agents_md)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, external_task_ref, project_config_id, worker_id, status as "status: WorkRunStatus", prompt_text,
                        repo_url, agents_md,
                        result_pr_url, result_exit_code, tokens_used, duration_ms,
                        created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>""#,
            id,
            &params.external_task_ref,
            params.project_config_id,
            &params.status as &WorkRunStatus,
            &params.prompt_text,
            &params.repo_url,
            &params.agents_md,
        )
        .fetch_one(db)
        .await
        .map_err(WorkRunsError::from)
    }

    pub async fn insert_work_run_if_not_active<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        params: InsertWorkRunParams,
    ) -> Result<bool, WorkRunsError> {
        let id = Uuid::new_v4();

        sqlx::query!(
            r#"INSERT INTO work_runs (id, external_task_ref, project_config_id, status, prompt_text, repo_url, agents_md)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT DO NOTHING"#,
            id,
            &params.external_task_ref,
            params.project_config_id,
            &params.status as &WorkRunStatus,
            &params.prompt_text,
            &params.repo_url,
            &params.agents_md,
        )
        .execute(db)
        .await
        .map(|result| result.rows_affected() > 0)
        .map_err(WorkRunsError::from)
    }

    pub async fn find_by_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<WorkRun, WorkRunsError> {
        sqlx::query_as!(
            WorkRun,
            r#"SELECT id, external_task_ref, project_config_id, worker_id, status as "status: WorkRunStatus",
             prompt_text, repo_url, agents_md, result_pr_url, result_exit_code, tokens_used, duration_ms,
             created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>"
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
    ) -> Result<Option<Uuid>, WorkRunsError> {
        sqlx::query_scalar!(
            r#"SELECT id FROM work_runs WHERE status = 'pending'::work_run_status ORDER BY created_at ASC LIMIT 1"#,
        )
        .fetch_optional(db)
        .await
        .map_err(WorkRunsError::from)
    }

    pub async fn list_all<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        status: Option<WorkRunStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WorkRunListItem>, WorkRunsError> {
        sqlx::query_as!(
            WorkRunListItem,
            r#"SELECT wr.id, wr.external_task_ref, wr.project_config_id, wr.worker_id,
             w.name as worker_name,
             wr.status as "status: WorkRunStatus", wr.prompt_text, wr.repo_url,
             wr.result_pr_url, wr.result_exit_code, wr.tokens_used, wr.duration_ms,
             wr.created_at as "created_at!: DateTime<Utc>"
             FROM work_runs wr LEFT JOIN workers w ON wr.worker_id = w.id
             WHERE ($1::work_run_status IS NULL OR wr.status = $1)
             ORDER BY wr.created_at DESC LIMIT $2 OFFSET $3"#,
            status as Option<WorkRunStatus>,
            limit,
            offset,
        )
        .fetch_all(db)
        .await
        .map_err(WorkRunsError::from)
    }

    pub async fn acknowledge<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        worker_id: Uuid,
    ) -> Result<WorkRun, WorkRunsError> {
        sqlx::query_as!(
            WorkRun,
            r#"UPDATE work_runs SET worker_id = $2, status = 'running'::work_run_status
             WHERE id = $1 AND status = 'pending'::work_run_status
             RETURNING id, external_task_ref, project_config_id, worker_id, status as "status: WorkRunStatus",
             prompt_text, repo_url, agents_md, result_pr_url, result_exit_code, tokens_used, duration_ms,
             created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>""#,
            id,
            worker_id,
        )
        .fetch_optional(db)
        .await
        .map_err(WorkRunsError::from)?
        .ok_or(WorkRunsError::AlreadyClaimed)
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
             duration_ms = $5, status = $6
             WHERE id = $1 AND status = 'running'::work_run_status
             RETURNING id, external_task_ref, project_config_id, worker_id, status as "status: WorkRunStatus",
             prompt_text, repo_url, agents_md, result_pr_url, result_exit_code, tokens_used, duration_ms,
             created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>""#,
            id,
            params.pr_url,
            params.exit_code,
            params.tokens_used,
            params.duration_ms,
            &params.status as &WorkRunStatus,
        )
        .fetch_optional(db)
        .await
        .map_err(WorkRunsError::from)?
        .ok_or(WorkRunsError::InvalidStatusTransition)
    }
}

pub struct SetResultParams<'a> {
    pub pr_url: &'a str,
    pub exit_code: i32,
    pub tokens_used: i64,
    pub duration_ms: i64,
    pub status: WorkRunStatus,
}
