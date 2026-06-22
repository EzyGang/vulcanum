use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    ChatGptAuthAttempt, CreateModelProviderRequest, ModelProviderConfig,
    UpdateModelProviderRequest, AUTH_TYPE_CHATGPT_OAUTH, OPENAI_PROVIDER_KEY,
};
use crate::services::model_providers::repository::{
    ensure_rows_affected, map_sqlx_error, CreateAuthAttemptParams, CreateOAuthProviderParams,
    ModelProvidersRepository,
};

const MODEL_PROVIDER_SELECT: &str = r#"
    SELECT id, team_id, provider_key, auth_type, display_name, credentials,
        oauth_credentials, oauth_metadata, created_at, updated_at
    FROM model_provider_configs
"#;

const AUTH_ATTEMPT_SELECT: &str = r#"
    SELECT id, team_id, user_id, device_code, user_code, verification_uri,
        interval_seconds, expires_at, status, error, created_at, updated_at
    FROM model_provider_auth_attempts
"#;

impl ModelProvidersRepository {
    pub async fn list_all<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
    ) -> Result<Vec<ModelProviderConfig>, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as::<_, ModelProviderConfig>(&format!(
            "{MODEL_PROVIDER_SELECT} WHERE team_id = $1 ORDER BY created_at DESC"
        ))
        .bind(team_id)
        .fetch_all(db)
        .await
        .map_err(ModelProvidersError::from)
    }

    pub async fn find_by_id<'c, Q>(
        &self,
        db: Q,
        id: Uuid,
        team_id: Uuid,
    ) -> Result<ModelProviderConfig, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as::<_, ModelProviderConfig>(&format!(
            "{MODEL_PROVIDER_SELECT} WHERE id = $1 AND team_id = $2"
        ))
        .bind(id)
        .bind(team_id)
        .fetch_optional(db)
        .await?
        .ok_or(ModelProvidersError::NotFound)
    }

    pub async fn find_by_provider_key<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        provider_key: &str,
    ) -> Result<ModelProviderConfig, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as::<_, ModelProviderConfig>(&format!(
            "{MODEL_PROVIDER_SELECT} WHERE team_id = $1 AND provider_key = $2 ORDER BY auth_type LIMIT 1"
        ))
        .bind(team_id)
        .bind(provider_key)
        .fetch_optional(db)
        .await?
        .ok_or(ModelProvidersError::NotFound)
    }

    pub async fn find_by_provider_auth<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        provider_key: &str,
        auth_type: &str,
    ) -> Result<ModelProviderConfig, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as::<_, ModelProviderConfig>(&format!(
            "{MODEL_PROVIDER_SELECT} WHERE team_id = $1 AND provider_key = $2 AND auth_type = $3"
        ))
        .bind(team_id)
        .bind(provider_key)
        .bind(auth_type)
        .fetch_optional(db)
        .await?
        .ok_or(ModelProvidersError::NotFound)
    }

    pub async fn find_by_ids<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        ids: &[Uuid],
    ) -> Result<Vec<ModelProviderConfig>, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as::<_, ModelProviderConfig>(&format!(
            "{MODEL_PROVIDER_SELECT} WHERE team_id = $1 AND id = ANY($2)"
        ))
        .bind(team_id)
        .bind(ids)
        .fetch_all(db)
        .await
        .map_err(ModelProvidersError::from)
    }

    pub async fn create<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        params: &CreateModelProviderRequest,
    ) -> Result<ModelProviderConfig, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, ModelProviderConfig>(
            r#"INSERT INTO model_provider_configs (
                id, team_id, provider_key, auth_type, display_name, credentials
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, team_id, provider_key, auth_type, display_name, credentials,
                oauth_credentials, oauth_metadata, created_at, updated_at"#,
        )
        .bind(id)
        .bind(team_id)
        .bind(&params.provider_key)
        .bind(&params.auth_type)
        .bind(&params.display_name)
        .bind(&params.credentials)
        .fetch_one(db)
        .await
        .map_err(map_sqlx_error)
    }

    pub async fn upsert_chatgpt_oauth<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        params: &CreateOAuthProviderParams<'_>,
    ) -> Result<ModelProviderConfig, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, ModelProviderConfig>(
            r#"INSERT INTO model_provider_configs (
                id, team_id, provider_key, auth_type, display_name, credentials,
                oauth_credentials, oauth_metadata
            ) VALUES ($1, $2, $3, $4, $5, '{}', $6, $7)
            ON CONFLICT (team_id, provider_key, auth_type) DO UPDATE SET
                display_name = EXCLUDED.display_name,
                oauth_credentials = EXCLUDED.oauth_credentials,
                oauth_metadata = EXCLUDED.oauth_metadata
            RETURNING id, team_id, provider_key, auth_type, display_name, credentials,
                oauth_credentials, oauth_metadata, created_at, updated_at"#,
        )
        .bind(id)
        .bind(team_id)
        .bind(OPENAI_PROVIDER_KEY)
        .bind(AUTH_TYPE_CHATGPT_OAUTH)
        .bind(params.display_name)
        .bind(params.oauth_credentials)
        .bind(params.oauth_metadata)
        .fetch_one(db)
        .await
        .map_err(map_sqlx_error)
    }

    pub async fn update<'c, Q>(
        &self,
        db: Q,
        id: Uuid,
        team_id: Uuid,
        params: &UpdateModelProviderRequest,
    ) -> Result<ModelProviderConfig, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as::<_, ModelProviderConfig>(
            r#"UPDATE model_provider_configs SET
              display_name = COALESCE($3, display_name),
              credentials = COALESCE($4, credentials)
              WHERE id = $1 AND team_id = $2
              RETURNING id, team_id, provider_key, auth_type, display_name, credentials,
                oauth_credentials, oauth_metadata, created_at, updated_at"#,
        )
        .bind(id)
        .bind(team_id)
        .bind(params.display_name.as_deref())
        .bind(params.credentials.as_ref())
        .fetch_optional(db)
        .await?
        .ok_or(ModelProvidersError::NotFound)
    }

    pub async fn delete<'c, Q>(
        &self,
        db: Q,
        id: Uuid,
        team_id: Uuid,
    ) -> Result<(), ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        let rows = sqlx::query("DELETE FROM model_provider_configs WHERE id = $1 AND team_id = $2")
            .bind(id)
            .bind(team_id)
            .execute(db)
            .await?
            .rows_affected();
        ensure_rows_affected(rows)
    }

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
        sqlx::query_as::<_, ChatGptAuthAttempt>(
            r#"INSERT INTO model_provider_auth_attempts (
                id, team_id, user_id, device_code, user_code, verification_uri,
                interval_seconds, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, team_id, user_id, device_code, user_code, verification_uri,
                interval_seconds, expires_at, status, error, created_at, updated_at"#,
        )
        .bind(id)
        .bind(team_id)
        .bind(params.user_id)
        .bind(params.device_code)
        .bind(params.user_code)
        .bind(params.verification_uri)
        .bind(params.interval_seconds)
        .bind(params.expires_at)
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
        sqlx::query_as::<_, ChatGptAuthAttempt>(&format!(
            "{AUTH_ATTEMPT_SELECT} WHERE id = $1 AND team_id = $2 AND user_id = $3"
        ))
        .bind(id)
        .bind(team_id)
        .bind(user_id)
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
        let rows = sqlx::query(
            "UPDATE model_provider_auth_attempts SET status = $2, error = $3 WHERE id = $1",
        )
        .bind(id)
        .bind(status)
        .bind(error)
        .execute(db)
        .await?
        .rows_affected();
        ensure_rows_affected(rows)
    }
}
