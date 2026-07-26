use super::{map_sqlx_error, Queryer, Uuid, WorkersError, WorkersRepository};

impl WorkersRepository {
    pub async fn decrement_active_jobs<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<(), WorkersError> {
        let row = sqlx::query!(
            r#"WITH worker AS (
                SELECT active_jobs FROM workers WHERE id = $1
            ),
            updated AS (
                UPDATE workers
                SET active_jobs = active_jobs - 1,
                    status = CASE WHEN active_jobs - 1 = 0 THEN 'idle'::worker_status ELSE status END
                WHERE id = $1 AND active_jobs > 0
                RETURNING active_jobs
            )
            SELECT EXISTS(SELECT 1 FROM worker) AS "worker_exists!",
             (SELECT active_jobs FROM worker) AS "previous_active_jobs?",
             (SELECT active_jobs FROM updated) AS "new_active_jobs?""#,
            id,
        )
        .fetch_one(db)
        .await
        .map_err(map_sqlx_error)?;

        if !row.worker_exists {
            return Err(WorkersError::WorkerNotFound);
        }

        let previous_active_jobs = row.previous_active_jobs.unwrap_or_default();
        if previous_active_jobs <= 0 || row.new_active_jobs.is_none() {
            return Err(WorkersError::ActiveJobsInvariant {
                worker_id: id,
                active_jobs: previous_active_jobs,
            });
        }

        Ok(())
    }

    pub async fn reset_active_jobs<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<(), WorkersError> {
        let rows = sqlx::query!(
            "UPDATE workers SET active_jobs = 0, status = 'idle'::worker_status WHERE id = $1",
            id,
        )
        .execute(db)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        if rows == 0 {
            return Err(WorkersError::WorkerNotFound);
        }

        Ok(())
    }

    pub async fn mark_stale_disconnected<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        threshold: chrono::Duration,
    ) -> Result<u64, WorkersError> {
        let cutoff = chrono::Utc::now() - threshold;

        sqlx::query!(
            r#"UPDATE workers SET status = 'disconnected'::worker_status, active_jobs = 0
             WHERE last_seen < $1 AND status != 'disconnected'::worker_status"#,
            cutoff,
        )
        .execute(db)
        .await
        .map(|result| result.rows_affected())
        .map_err(map_sqlx_error)
    }
}
