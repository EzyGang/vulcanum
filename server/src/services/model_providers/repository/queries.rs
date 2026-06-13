use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    CreateModelProviderRequest, ModelProviderConfig, UpdateModelProviderRequest,
};
use crate::services::model_providers::repository::ModelProvidersRepository;

impl ModelProvidersRepository {
    pub async fn list_all<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        team_id: Uuid,
    ) -> Result<Vec<ModelProviderConfig>, ModelProvidersError> {
        sqlx::query_as::<_, ModelProviderConfig>(
            "SELECT id, team_id, provider_key, display_name, credentials, advanced_options, created_at, updated_at
             FROM model_provider_configs WHERE team_id = $1 ORDER BY created_at DESC",
        )
        .bind(team_id)
        .fetch_all(db)
        .await
        .map_err(ModelProvidersError::from)
    }

    pub async fn find_by_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        team_id: Uuid,
    ) -> Result<ModelProviderConfig, ModelProvidersError> {
        sqlx::query_as::<_, ModelProviderConfig>(
            "SELECT id, team_id, provider_key, display_name, credentials, advanced_options, created_at, updated_at
             FROM model_provider_configs WHERE id = $1 AND team_id = $2",
        )
        .bind(id)
        .bind(team_id)
        .fetch_optional(db)
        .await?
        .ok_or(ModelProvidersError::NotFound)
    }

    pub async fn find_by_provider_key<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        team_id: Uuid,
        provider_key: &str,
    ) -> Result<ModelProviderConfig, ModelProvidersError> {
        sqlx::query_as::<_, ModelProviderConfig>(
            "SELECT id, team_id, provider_key, display_name, credentials, advanced_options, created_at, updated_at
             FROM model_provider_configs WHERE team_id = $1 AND provider_key = $2",
        )
        .bind(team_id)
        .bind(provider_key)
        .fetch_optional(db)
        .await?
        .ok_or(ModelProvidersError::NotFound)
    }

    pub async fn create<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        team_id: Uuid,
        params: &CreateModelProviderRequest,
    ) -> Result<ModelProviderConfig, ModelProvidersError> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, ModelProviderConfig>(
            r#"INSERT INTO model_provider_configs (id, team_id, provider_key, display_name, credentials, advanced_options)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, team_id, provider_key, display_name, credentials, advanced_options, created_at, updated_at"#,
        )
        .bind(id)
        .bind(team_id)
        .bind(&params.provider_key)
        .bind(&params.display_name)
        .bind(&params.credentials)
        .bind(&params.advanced_options)
        .fetch_one(db)
        .await
        .map_err(map_sqlx_error)
    }

    pub async fn update<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        team_id: Uuid,
        params: &UpdateModelProviderRequest,
    ) -> Result<ModelProviderConfig, ModelProvidersError> {
        sqlx::query_as::<_, ModelProviderConfig>(
            r#"UPDATE model_provider_configs SET
             display_name = COALESCE($3, display_name),
             credentials = COALESCE($4, credentials),
             advanced_options = COALESCE($5, advanced_options)
             WHERE id = $1 AND team_id = $2
             RETURNING id, team_id, provider_key, display_name, credentials, advanced_options, created_at, updated_at"#,
        )
        .bind(id)
        .bind(team_id)
        .bind(params.display_name.as_deref())
        .bind(params.credentials.as_ref())
        .bind(params.advanced_options.as_ref())
        .fetch_optional(db)
        .await?
        .ok_or(ModelProvidersError::NotFound)
    }

    pub async fn delete<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        team_id: Uuid,
    ) -> Result<(), ModelProvidersError> {
        let rows = sqlx::query("DELETE FROM model_provider_configs WHERE id = $1 AND team_id = $2")
            .bind(id)
            .bind(team_id)
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
