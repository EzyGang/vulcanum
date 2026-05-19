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

        sqlx::query_as::<_, Worker>(
            "INSERT INTO workers (id, name, refresh_token_hash, refresh_expires_at, status, capabilities) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, name, refresh_token_hash, refresh_expires_at, last_seen, status::text as status, capabilities, created_at",
        )
        .bind(id)
        .bind(name)
        .bind(refresh_token_hash)
        .bind(refresh_expires_at)
        .bind(WorkerStatus::Idle)
        .bind(capabilities)
        .fetch_one(db)
        .await
        .map_err(map_sqlx_error)
    }

    #[allow(dead_code)]
    pub async fn find_by_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<Worker, WorkersError> {
        sqlx::query_as::<_, Worker>(
            "SELECT id, name, refresh_token_hash, refresh_expires_at, last_seen, status::text as status, capabilities, created_at \
             FROM workers WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(WorkersError::WorkerNotFound)
    }

    pub async fn find_by_refresh_token_hash<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        hash: &str,
    ) -> Result<Worker, WorkersError> {
        sqlx::query_as::<_, Worker>(
            "SELECT id, name, refresh_token_hash, refresh_expires_at, last_seen, status::text as status, capabilities, created_at \
             FROM workers WHERE refresh_token_hash = $1",
        )
        .bind(hash)
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
        sqlx::query_as::<_, Worker>(
            "UPDATE workers SET refresh_token_hash = $1, refresh_expires_at = $2 \
             WHERE id = $3 \
             RETURNING id, name, refresh_token_hash, refresh_expires_at, last_seen, status::text as status, capabilities, created_at",
        )
        .bind(new_hash)
        .bind(new_expires_at)
        .bind(worker_id)
        .fetch_optional(db)
        .await?
        .ok_or(WorkersError::WorkerNotFound)
    }

    #[allow(dead_code)]
    pub async fn update_last_seen<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        ts: DateTime<Utc>,
    ) -> Result<(), WorkersError> {
        sqlx::query("UPDATE workers SET last_seen = $1 WHERE id = $2")
            .bind(ts)
            .bind(id)
            .execute(db)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn list_all<'c, Q: Queryer<'c>>(&self, db: Q) -> Result<Vec<Worker>, WorkersError> {
        sqlx::query_as::<_, Worker>(
            "SELECT id, name, refresh_token_hash, refresh_expires_at, last_seen, status::text as status, capabilities, created_at \
             FROM workers ORDER BY created_at DESC",
        )
        .fetch_all(db)
        .await
        .map_err(map_sqlx_error)
    }

    pub async fn delete<'c, Q: Queryer<'c>>(&self, db: Q, id: Uuid) -> Result<(), WorkersError> {
        let rows = sqlx::query("DELETE FROM workers WHERE id = $1")
            .bind(id)
            .execute(db)
            .await
            .map_err(map_sqlx_error)?
            .rows_affected();

        if rows == 0 {
            return Err(WorkersError::WorkerNotFound);
        }

        Ok(())
    }
}
