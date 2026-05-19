use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::services::workers::errors::WorkersError;
use crate::services::workers::model::Worker;
use crate::services::workers::repository::{map_sqlx_error, Queryer, WorkersRepository};

const SELECT_COLUMNS: &str =
    "id, name, refresh_token_hash, last_seen, status::text as status, capabilities, created_at";

impl WorkersRepository {
    pub async fn create<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        name: &str,
        refresh_token_hash: &str,
        capabilities: &Value,
    ) -> Result<Worker, WorkersError> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, Worker>(&format!(
            "INSERT INTO workers (id, name, refresh_token_hash, status, capabilities) \
             VALUES ($1, $2, $3, 'idle', $4) \
             RETURNING {}",
            SELECT_COLUMNS
        ))
        .bind(id)
        .bind(name)
        .bind(refresh_token_hash)
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
        sqlx::query_as::<_, Worker>(&format!(
            "SELECT {} FROM workers WHERE id = $1",
            SELECT_COLUMNS
        ))
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
        sqlx::query_as::<_, Worker>(&format!(
            "SELECT {} FROM workers WHERE refresh_token_hash = $1",
            SELECT_COLUMNS
        ))
        .bind(hash)
        .fetch_optional(db)
        .await?
        .ok_or(WorkersError::InvalidRefreshToken)
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
        sqlx::query_as::<_, Worker>(&format!(
            "SELECT {} FROM workers ORDER BY created_at DESC",
            SELECT_COLUMNS
        ))
        .fetch_all(db)
        .await
        .map_err(map_sqlx_error)
    }

    #[allow(dead_code)]
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
