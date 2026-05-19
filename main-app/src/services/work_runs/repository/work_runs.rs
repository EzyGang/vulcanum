use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::{WorkRun, WorkRunStatus};
use crate::services::work_runs::repository::WorkRunsRepository;

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
}
