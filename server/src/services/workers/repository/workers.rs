use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::services::workers::errors::WorkersError;
use crate::services::workers::model::{Worker, WorkerStatus};
use crate::services::workers::repository::{map_sqlx_error, Queryer, WorkersRepository};

impl WorkersRepository {
    pub async fn create<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        name: &str,
        refresh_token_hash: &str,
        refresh_expires_at: DateTime<Utc>,
        capabilities: &serde_json::Value,
    ) -> Result<Worker, WorkersError> {
        let id = Uuid::new_v4();

        sqlx::query_as!(
            Worker,
            r#"INSERT INTO workers (id, name, refresh_token_hash, refresh_expires_at, status, capabilities, active_jobs, max_concurrent_jobs, consecutive_errors)
             VALUES ($1, $2, $3, $4, $5::worker_status, $6, 0, $7, 0)
             RETURNING id, name, refresh_token_hash, refresh_expires_at, last_seen,
             status as "status: WorkerStatus", capabilities, created_at as "created_at!: DateTime<Utc>",
             active_jobs, max_concurrent_jobs, consecutive_errors"#,
            id,
            name,
            refresh_token_hash,
            refresh_expires_at,
            WorkerStatus::Idle as WorkerStatus,
            capabilities,
            crate::services::workers::model::DEFAULT_MAX_CONCURRENT_JOBS,
        )
        .fetch_one(db)
        .await
        .map_err(map_sqlx_error)
    }

    pub async fn find_by_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<Worker, WorkersError> {
        sqlx::query_as!(
            Worker,
            r#"SELECT id, name, refresh_token_hash, refresh_expires_at, last_seen,
             status as "status: WorkerStatus", capabilities, created_at as "created_at!: DateTime<Utc>",
             active_jobs, max_concurrent_jobs, consecutive_errors
             FROM workers WHERE id = $1"#,
            id,
        )
        .fetch_optional(db)
        .await?
        .ok_or(WorkersError::WorkerNotFound)
    }

    pub async fn find_by_refresh_token_hash<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        hash: &str,
    ) -> Result<Worker, WorkersError> {
        sqlx::query_as!(
            Worker,
            r#"SELECT id, name, refresh_token_hash, refresh_expires_at, last_seen,
             status as "status: WorkerStatus", capabilities, created_at as "created_at!: DateTime<Utc>",
             active_jobs, max_concurrent_jobs, consecutive_errors
             FROM workers WHERE refresh_token_hash = $1"#,
            hash,
        )
        .fetch_optional(db)
        .await?
        .ok_or(WorkersError::InvalidRefreshToken)
    }

    pub async fn update_refresh_token<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        worker_id: Uuid,
        new_hash: &str,
        new_expires_at: DateTime<Utc>,
    ) -> Result<Worker, WorkersError> {
        sqlx::query_as!(
            Worker,
            r#"UPDATE workers SET refresh_token_hash = $1, refresh_expires_at = $2
             WHERE id = $3
             RETURNING id, name, refresh_token_hash, refresh_expires_at, last_seen,
             status as "status: WorkerStatus", capabilities, created_at as "created_at!: DateTime<Utc>",
             active_jobs, max_concurrent_jobs, consecutive_errors"#,
            new_hash,
            new_expires_at,
            worker_id,
        )
        .fetch_optional(db)
        .await?
        .ok_or(WorkersError::WorkerNotFound)
    }

    pub async fn update_last_seen<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        ts: DateTime<Utc>,
    ) -> Result<(), WorkersError> {
        sqlx::query!("UPDATE workers SET last_seen = $1 WHERE id = $2", ts, id)
            .execute(db)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    pub async fn list_all<'c, Q: Queryer<'c>>(&self, db: Q) -> Result<Vec<Worker>, WorkersError> {
        sqlx::query_as!(
            Worker,
            r#"SELECT id, name, refresh_token_hash, refresh_expires_at, last_seen,
             status as "status: WorkerStatus", capabilities, created_at as "created_at!: DateTime<Utc>",
             active_jobs, max_concurrent_jobs, consecutive_errors
             FROM workers ORDER BY created_at DESC"#,
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

    pub async fn increment_active_jobs<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<(), WorkersError> {
        sqlx::query!(
            "UPDATE workers SET active_jobs = active_jobs + 1, status = 'busy'::worker_status
             WHERE id = $1 AND active_jobs < max_concurrent_jobs",
            id,
        )
        .execute(db)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    pub async fn decrement_active_jobs<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<(), WorkersError> {
        sqlx::query!(
            "UPDATE workers SET active_jobs = active_jobs - 1, status = CASE WHEN active_jobs - 1 = 0 THEN 'idle'::worker_status ELSE status END WHERE id = $1 AND active_jobs > 0",
            id,
        )
        .execute(db)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    pub async fn reset_active_jobs<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<(), WorkersError> {
        sqlx::query!(
            "UPDATE workers SET active_jobs = 0, status = 'idle'::worker_status WHERE id = $1",
            id,
        )
        .execute(db)
        .await
        .map_err(map_sqlx_error)?;
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
