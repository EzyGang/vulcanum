use super::{
    map_sqlx_error, DateTime, Queryer, Utc, Uuid, Worker, WorkerStatus, WorkersError,
    WorkersRepository,
};

impl WorkersRepository {
    pub async fn update_last_seen<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        ts: DateTime<Utc>,
    ) -> Result<(), WorkersError> {
        sqlx::query!(
            r#"UPDATE workers SET last_seen = $1,
             status = CASE
                 WHEN status = 'disconnected'::worker_status THEN 'idle'::worker_status
                 ELSE status
             END
             WHERE id = $2"#,
            ts,
            id,
        )
        .execute(db)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    pub async fn list_all<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        team_id: Uuid,
    ) -> Result<Vec<Worker>, WorkersError> {
        sqlx::query_as!(
            Worker,
            r#"SELECT id, team_id, name, refresh_token_hash, refresh_expires_at, last_seen,
             status as "status: WorkerStatus", capabilities, created_at as "created_at!: DateTime<Utc>",
             active_jobs, max_concurrent_jobs, consecutive_errors
              FROM workers WHERE team_id = $1 ORDER BY created_at DESC"#,
            team_id,
        )
        .fetch_all(db)
        .await
        .map_err(map_sqlx_error)
    }

    pub async fn delete<'c, Q: Queryer<'c>>(&self, db: Q, id: Uuid) -> Result<(), WorkersError> {
        let rows = sqlx::query!("DELETE FROM workers WHERE id = $1", id)
            .execute(db)
            .await
            .map_err(map_sqlx_error)?
            .rows_affected();

        if rows == 0 {
            return Err(WorkersError::WorkerNotFound);
        }

        Ok(())
    }

    pub async fn set_status<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        status: WorkerStatus,
    ) -> Result<(), WorkersError> {
        let rows = sqlx::query!(
            "UPDATE workers SET status = $1::worker_status WHERE id = $2",
            status as WorkerStatus,
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

    pub async fn set_status_and_reset<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        status: WorkerStatus,
    ) -> Result<(), WorkersError> {
        let rows = sqlx::query!(
            r#"UPDATE workers SET status = $1::worker_status, active_jobs = 0, consecutive_errors = 0
             WHERE id = $2"#,
            status as WorkerStatus,
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

    pub async fn increment_consecutive_errors<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        threshold: i32,
    ) -> Result<i32, WorkersError> {
        let row = sqlx::query!(
            r#"UPDATE workers SET consecutive_errors = consecutive_errors + 1,
             status = CASE WHEN consecutive_errors + 1 >= $2 THEN 'unhealthy'::worker_status ELSE status END
             WHERE id = $1
             RETURNING consecutive_errors"#,
            id,
            threshold,
        )
        .fetch_optional(db)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(r) => Ok(r.consecutive_errors),
            None => Err(WorkersError::WorkerNotFound),
        }
    }

    pub async fn reset_active_jobs_only<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<(), WorkersError> {
        let rows = sqlx::query!("UPDATE workers SET active_jobs = 0 WHERE id = $1", id,)
            .execute(db)
            .await
            .map_err(map_sqlx_error)?
            .rows_affected();

        if rows == 0 {
            return Err(WorkersError::WorkerNotFound);
        }

        Ok(())
    }

    pub async fn reset_consecutive_errors<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<(), WorkersError> {
        sqlx::query!(
            "UPDATE workers SET consecutive_errors = 0 WHERE id = $1",
            id,
        )
        .execute(db)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }
}
