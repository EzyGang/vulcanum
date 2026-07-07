use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;

impl WorkRunsRepository {
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
            r#"WITH candidates AS (
                SELECT id, worker_id
                FROM work_runs
                WHERE status = 'running'::work_run_status
                AND updated_at < NOW() - INTERVAL '1 second' * $1
                AND finish_blocked_reason IS NULL
            ),
            reset_runs AS (
                UPDATE work_runs wr
                SET status = 'pending'::work_run_status, worker_id = NULL
                FROM candidates c
                WHERE wr.id = c.id
                RETURNING c.worker_id
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

    async fn reset_worker_active_jobs_raw<'c, Q: Queryer<'c>>(
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
}
