use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::{WorkRun, WorkRunStatus};
use crate::services::work_runs::repository::WorkRunsRepository;

fn map_err(err: sqlx::Error) -> WorkRunsError {
    WorkRunsError::from(err)
}

#[allow(dead_code)]
pub struct InsertWorkRunParams {
    pub external_task_ref: String,
    pub project_config_id: Uuid,
    pub prompt_text: String,
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
            r#"INSERT INTO work_runs (id, external_task_ref, project_config_id, status, prompt_text)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, external_task_ref, project_config_id, worker_id, status as "status: WorkRunStatus", prompt_text,
                        result_pr_url, result_exit_code, tokens_used, duration_ms,
                        created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>""#,
            id,
            &params.external_task_ref,
            params.project_config_id,
            &params.status as &WorkRunStatus,
            &params.prompt_text,
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
            r#"INSERT INTO work_runs (id, external_task_ref, project_config_id, status, prompt_text)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING"#,
            id,
            &params.external_task_ref,
            params.project_config_id,
            &params.status as &WorkRunStatus,
            &params.prompt_text,
        )
        .execute(db)
        .await
        .map(|result| result.rows_affected() > 0)
        .map_err(WorkRunsError::from)
    }

    #[allow(dead_code)]
    pub async fn find_by_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<WorkRun, WorkRunsError> {
        sqlx::query_as!(
            WorkRun,
            r#"SELECT id, external_task_ref, project_config_id, worker_id, status as "status: WorkRunStatus",
             prompt_text, result_pr_url, result_exit_code, tokens_used, duration_ms,
             created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>"
             FROM work_runs WHERE id = $1"#,
            id,
        )
        .fetch_optional(db)
        .await
        .map_err(map_err)?
        .ok_or(WorkRunsError::NotFound)
    }

    #[allow(dead_code)]
    pub async fn find_oldest_pending_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<Option<Uuid>, WorkRunsError> {
        sqlx::query_scalar!(
            r#"SELECT id FROM work_runs WHERE status = 'pending'::work_run_status ORDER BY created_at ASC LIMIT 1"#,
        )
        .fetch_optional(db)
        .await
        .map_err(map_err)
    }

    #[allow(dead_code)]
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
             prompt_text, result_pr_url, result_exit_code, tokens_used, duration_ms,
             created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>""#,
            id,
            worker_id,
        )
        .fetch_optional(db)
        .await
        .map_err(map_err)?
        .ok_or(WorkRunsError::AlreadyClaimed)
    }

    #[allow(dead_code)]
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
             prompt_text, result_pr_url, result_exit_code, tokens_used, duration_ms,
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
        .map_err(map_err)?
        .ok_or(WorkRunsError::InvalidStatusTransition)
    }
}

pub struct SetResultParams<'a> {
    pub pr_url: &'a str,
    pub exit_code: i32,
    pub tokens_used: i32,
    pub duration_ms: i32,
    pub status: WorkRunStatus,
}
