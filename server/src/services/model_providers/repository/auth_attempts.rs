use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::ChatGptAuthAttempt;
use crate::services::model_providers::repository::{
    CreateAuthAttemptParams, ModelProvidersRepository,
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

    pub async fn fail_pending_auth_attempts_for_user<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        user_id: &str,
        error: &str,
    ) -> Result<(), ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query!(
            r#"UPDATE model_provider_auth_attempts
               SET status = 'failed', error = $3
               WHERE team_id = $1 AND user_id = $2 AND status = 'pending'"#,
            team_id,
            user_id,
            error,
        )
        .execute(db)
        .await?;
        Ok(())
    }

    pub async fn try_update_auth_attempt_status_from<'c, Q>(
        &self,
        db: Q,
        id: Uuid,
        expected_status: &str,
        status: &str,
        error: Option<&str>,
    ) -> Result<bool, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        let rows = sqlx::query!(
            r#"UPDATE model_provider_auth_attempts
               SET status = $3, error = $4
               WHERE id = $1 AND status = $2"#,
            id,
            expected_status,
            status,
            error,
        )
        .execute(db)
        .await?
        .rows_affected();
        Ok(rows > 0)
    }

    pub async fn try_update_auth_attempt_interval_from<'c, Q>(
        &self,
        db: Q,
        id: Uuid,
        expected_status: &str,
        interval_seconds: i32,
    ) -> Result<bool, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        let rows = sqlx::query!(
            r#"UPDATE model_provider_auth_attempts
               SET interval_seconds = $3
               WHERE id = $1 AND status = $2"#,
            id,
            expected_status,
            interval_seconds,
        )
        .execute(db)
        .await?
        .rows_affected();
        Ok(rows > 0)
    }
}
