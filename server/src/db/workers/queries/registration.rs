use super::*;

impl WorkersRepository {
    pub async fn create<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        params: CreateWorkerParams<'_>,
    ) -> Result<Worker, WorkersError> {
        let id = Uuid::new_v4();

        sqlx::query_as!(
            Worker,
            r#"INSERT INTO workers (id, team_id, name, refresh_token_hash, refresh_expires_at, status, capabilities, active_jobs, max_concurrent_jobs, consecutive_errors)
             VALUES ($1, $2, $3, $4, $5, $6::worker_status, $7, 0, $8, 0)
             RETURNING id, team_id, name, refresh_token_hash, refresh_expires_at, last_seen,
             status as "status: WorkerStatus", capabilities, created_at as "created_at!: DateTime<Utc>",
            active_jobs, max_concurrent_jobs, consecutive_errors"#,
            id,
            params.team_id,
            params.name,
            params.refresh_token_hash,
            params.refresh_expires_at,
            WorkerStatus::Idle as WorkerStatus,
            params.capabilities,
            params.max_concurrent_jobs,
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
            r#"SELECT id, team_id, name, refresh_token_hash, refresh_expires_at, last_seen,
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
            r#"SELECT id, team_id, name, refresh_token_hash, refresh_expires_at, last_seen,
             status as "status: WorkerStatus", capabilities, created_at as "created_at!: DateTime<Utc>",
             active_jobs, max_concurrent_jobs, consecutive_errors
             FROM workers WHERE refresh_token_hash = $1"#,
            hash,
        )
        .fetch_optional(db)
        .await?
        .ok_or(WorkersError::InvalidRefreshToken)
    }

    pub async fn rotate_refresh_token<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        old_hash: &str,
        new_hash: &str,
        new_expires_at: DateTime<Utc>,
    ) -> Result<Worker, WorkersError> {
        sqlx::query_as!(
            Worker,
            r#"UPDATE workers SET refresh_token_hash = $1, refresh_expires_at = $2
             WHERE refresh_token_hash = $3 AND refresh_expires_at >= NOW()
              RETURNING id, team_id, name, refresh_token_hash, refresh_expires_at, last_seen,
             status as "status: WorkerStatus", capabilities, created_at as "created_at!: DateTime<Utc>",
             active_jobs, max_concurrent_jobs, consecutive_errors"#,
            new_hash,
            new_expires_at,
            old_hash,
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
              RETURNING id, team_id, name, refresh_token_hash, refresh_expires_at, last_seen,
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
}
