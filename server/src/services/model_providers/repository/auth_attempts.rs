use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::ChatGptAuthAttempt;
use crate::services::model_providers::repository::{
    ensure_rows_affected, CreateAuthAttemptParams, ModelProvidersRepository,
};

impl ModelProvidersRepository {
    pub async fn create_auth_attempt<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        params: &CreateAuthAttemptParams<'_>,
    ) -> Result<ChatGptAuthAttempt, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        let id = Uuid::new_v4();
        sqlx::query_as!(
            ChatGptAuthAttempt,
            r#"INSERT INTO model_provider_auth_attempts (
                id, team_id, user_id, encrypted_device_code, user_code, verification_uri,
                display_name, interval_seconds, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, team_id, user_id, encrypted_device_code, user_code, verification_uri,
                display_name, interval_seconds, expires_at, status, error, created_at, updated_at"#,
            id,
            team_id,
            params.user_id,
            params.encrypted_device_code,
            params.user_code,
            params.verification_uri,
            params.display_name,
            params.interval_seconds,
            params.expires_at,
        )
        .fetch_one(db)
        .await
        .map_err(ModelProvidersError::from)
    }

    pub async fn find_auth_attempt<'c, Q>(
        &self,
        db: Q,
        id: Uuid,
        team_id: Uuid,
        user_id: &str,
    ) -> Result<ChatGptAuthAttempt, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            ChatGptAuthAttempt,
            r#"SELECT id, team_id, user_id, encrypted_device_code, user_code, verification_uri,
                display_name, interval_seconds, expires_at, status, error, created_at, updated_at
              FROM model_provider_auth_attempts
              WHERE id = $1 AND team_id = $2 AND user_id = $3"#,
            id,
            team_id,
            user_id,
        )
        .fetch_optional(db)
        .await?
        .ok_or(ModelProvidersError::NotFound)
    }

    pub async fn update_auth_attempt_status<'c, Q>(
        &self,
        db: Q,
        id: Uuid,
        status: &str,
        error: Option<&str>,
    ) -> Result<(), ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        let rows = sqlx::query!(
            "UPDATE model_provider_auth_attempts SET status = $2, error = $3 WHERE id = $1",
            id,
            status,
            error,
        )
        .execute(db)
        .await?
        .rows_affected();
        ensure_rows_affected(rows)
    }

    pub async fn update_auth_attempt_interval<'c, Q>(
        &self,
        db: Q,
        id: Uuid,
        interval_seconds: i32,
    ) -> Result<(), ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        let rows = sqlx::query!(
            "UPDATE model_provider_auth_attempts SET interval_seconds = $2 WHERE id = $1",
            id,
            interval_seconds,
        )
        .execute(db)
        .await?
        .rows_affected();
        ensure_rows_affected(rows)
    }
}
