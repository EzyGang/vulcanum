use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    CreateModelProviderRequest, ModelProviderConfig, UpdateModelProviderRequest,
};
use crate::services::model_providers::repository::ModelProvidersRepository;

impl ModelProvidersRepository {
    pub async fn list_all<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
    ) -> Result<Vec<ModelProviderConfig>, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            ModelProviderConfig,
            "SELECT id, team_id, provider_key, display_name, credentials, created_at, updated_at
              FROM model_provider_configs WHERE team_id = $1 ORDER BY created_at DESC",
            team_id,
        )
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
        sqlx::query_as!(
            ModelProviderConfig,
            "SELECT id, team_id, provider_key, display_name, credentials, created_at, updated_at
              FROM model_provider_configs WHERE id = $1 AND team_id = $2",
            id,
            team_id,
        )
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
        sqlx::query_as!(
            ModelProviderConfig,
            "SELECT id, team_id, provider_key, display_name, credentials, created_at, updated_at
              FROM model_provider_configs WHERE team_id = $1 AND provider_key = $2",
            team_id,
            provider_key,
        )
        .fetch_optional(db)
        .await?
        .ok_or(ModelProvidersError::NotFound)
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
        sqlx::query_as!(
            ModelProviderConfig,
            r#"INSERT INTO model_provider_configs (id, team_id, provider_key, display_name, credentials)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, team_id, provider_key, display_name, credentials, created_at, updated_at"#,
            id,
            team_id,
            params.provider_key,
            params.display_name,
            params.credentials,
        )
        .fetch_one(db)
        .await
        .map_err(map_sqlx_error)
    }

    pub async fn upsert_by_provider_key<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        provider_key: &str,
        display_name: &str,
        credentials: &serde_json::Value,
    ) -> Result<ModelProviderConfig, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        let id = Uuid::new_v4();
        sqlx::query_as!(
            ModelProviderConfig,
            r#"INSERT INTO model_provider_configs (id, team_id, provider_key, display_name, credentials)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (team_id, provider_key) DO UPDATE SET
               display_name = EXCLUDED.display_name,
               credentials = EXCLUDED.credentials
             RETURNING id, team_id, provider_key, display_name, credentials, created_at, updated_at"#,
            id,
            team_id,
            provider_key,
            display_name,
            credentials,
        )
        .fetch_one(db)
        .await
        .map_err(ModelProvidersError::from)
    }

    pub async fn update_credentials<'c, Q>(
        &self,
        db: Q,
        id: Uuid,
        team_id: Uuid,
        credentials: &serde_json::Value,
    ) -> Result<ModelProviderConfig, ModelProvidersError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            ModelProviderConfig,
            r#"UPDATE model_provider_configs SET credentials = $3
              WHERE id = $1 AND team_id = $2
              RETURNING id, team_id, provider_key, display_name, credentials, created_at, updated_at"#,
            id,
            team_id,
            credentials,
        )
        .fetch_optional(db)
        .await?
        .ok_or(ModelProvidersError::NotFound)
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
        sqlx::query_as!(
            ModelProviderConfig,
            r#"UPDATE model_provider_configs SET
              display_name = COALESCE($3, display_name),
              credentials = COALESCE($4, credentials)
              WHERE id = $1 AND team_id = $2
              RETURNING id, team_id, provider_key, display_name, credentials, created_at, updated_at"#,
            id,
            team_id,
            params.display_name.as_deref(),
            params.credentials.as_ref(),
        )
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
        let rows = sqlx::query!(
            "DELETE FROM model_provider_configs WHERE id = $1 AND team_id = $2",
            id,
            team_id,
        )
        .execute(db)
        .await?
        .rows_affected();
        if rows == 0 {
            return Err(ModelProvidersError::NotFound);
        }
        Ok(())
    }
}

fn map_sqlx_error(err: sqlx::Error) -> ModelProvidersError {
    let is_duplicate = err
        .as_database_error()
        .map(|db_err| db_err.constraint() == Some("model_provider_configs_team_provider_key"))
        .unwrap_or(false);
    if is_duplicate {
        ModelProvidersError::DuplicateProvider
    } else {
        ModelProvidersError::Database(err)
    }
}
