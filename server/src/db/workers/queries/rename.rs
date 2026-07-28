use super::{
    map_sqlx_error, DateTime, Queryer, Utc, Uuid, Worker, WorkerStatus, WorkersError,
    WorkersRepository,
};

impl WorkersRepository {
    pub async fn rename<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        name: &str,
    ) -> Result<Worker, WorkersError> {
        sqlx::query_as!(
            Worker,
            r#"UPDATE workers SET name = $1
               WHERE id = $2
               RETURNING id, team_id, name, refresh_token_hash, refresh_expires_at, last_seen,
               status as "status: WorkerStatus", capabilities, created_at as "created_at!: DateTime<Utc>",
               active_jobs, max_concurrent_jobs, consecutive_errors"#,
            name,
            id,
        )
        .fetch_optional(db)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(WorkersError::WorkerNotFound)
    }
}
