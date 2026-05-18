use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::WorkRun;
use crate::services::work_runs::repository::WorkRunsRepository;

#[allow(dead_code)]
pub trait Queryer<'c>: Executor<'c, Database = Postgres> {}

impl<'c> Queryer<'c> for &sqlx::PgPool {}

impl<'c> Queryer<'c> for &'c mut sqlx::PgConnection {}

#[allow(dead_code)]
pub struct InsertWorkRunParams {
    pub external_task_ref: String,
    pub project_config_id: Uuid,
    pub prompt_text: String,
    pub status: String,
}

impl WorkRunsRepository {
    #[allow(dead_code)]
    pub async fn insert_work_run<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        params: InsertWorkRunParams,
    ) -> Result<WorkRun, WorkRunsError> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, WorkRun>(
             "INSERT INTO work_runs (id, external_task_ref, project_config_id, status, prompt_text) \
             VALUES ($1, $2, $3, $4::work_run_status, $5) \
             RETURNING id, external_task_ref, project_config_id, worker_id, status::text, prompt_text, \
                       result_pr_url, result_exit_code, tokens_used, duration_ms, created_at, updated_at",
        )
        .bind(id)
        .bind(&params.external_task_ref)
        .bind(params.project_config_id)
        .bind(&params.status)
        .bind(&params.prompt_text)
        .fetch_one(db)
        .await
        .map_err(WorkRunsError::from)
    }
}
