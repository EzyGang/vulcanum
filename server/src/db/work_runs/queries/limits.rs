use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;

impl WorkRunsRepository {
    pub async fn count_active_implementations_by_project<'c, Q>(
        &self,
        db: Q,
        project_config_id: Uuid,
    ) -> Result<i64, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!: i64" FROM work_runs
             WHERE project_config_id = $1
               AND work_type = 'implementation'::work_run_type
               AND status IN ('pending'::work_run_status, 'dispatched'::work_run_status, 'running'::work_run_status)"#,
            project_config_id,
        )
        .fetch_one(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(count)
    }
}
