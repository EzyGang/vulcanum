use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::provider_configs::errors::IntegrationProvidersError;
use crate::services::provider_configs::model::{CreateProviderRequest, IntegrationProvider};
use crate::services::provider_configs::repository::IntegrationProvidersRepository;
use crate::services::providers::model::IntegrationType;

pub struct UpdateProviderParams<'a> {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: Option<&'a str>,
    pub provider_type: Option<IntegrationType>,
    pub instance_url: Option<&'a str>,
    pub api_key: Option<&'a str>,
}

impl IntegrationProvidersRepository {
    pub async fn list_all<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        team_id: Uuid,
    ) -> Result<Vec<IntegrationProvider>, IntegrationProvidersError> {
        sqlx::query_as!(
            IntegrationProvider,
            r#"SELECT id, team_id, name, provider_type as "provider_type!: _", instance_url, api_key, created_at as "created_at!: chrono::DateTime<chrono::Utc>"
             FROM integration_providers WHERE team_id = $1 ORDER BY created_at DESC"#,
            team_id,
        )
        .fetch_all(db)
        .await
        .map_err(IntegrationProvidersError::from)
    }

    pub async fn find_by_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        team_id: Uuid,
    ) -> Result<IntegrationProvider, IntegrationProvidersError> {
        sqlx::query_as!(
            IntegrationProvider,
            r#"SELECT id, team_id, name, provider_type as "provider_type!: _", instance_url, api_key, created_at as "created_at!: chrono::DateTime<chrono::Utc>"
             FROM integration_providers WHERE id = $1 AND team_id = $2"#,
            id,
            team_id,
        )
        .fetch_optional(db)
        .await?
        .ok_or(IntegrationProvidersError::NotFound)
    }

    pub async fn create<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        team_id: Uuid,
        params: &CreateProviderRequest,
    ) -> Result<IntegrationProvider, IntegrationProvidersError> {
        let id = Uuid::new_v4();
        let provider_type = params.provider_type.unwrap_or_default();

        sqlx::query_as!(
            IntegrationProvider,
            r#"INSERT INTO integration_providers (id, team_id, name, provider_type, instance_url, api_key)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, team_id, name, provider_type as "provider_type!: _", instance_url, api_key, created_at as "created_at!: chrono::DateTime<chrono::Utc>""#,
            id,
            team_id,
            &params.name,
            provider_type as _,
            &params.instance_url,
            &params.api_key,
        )
        .fetch_one(db)
        .await
        .map_err(map_sqlx_error)
    }

    pub async fn update<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        params: UpdateProviderParams<'_>,
    ) -> Result<IntegrationProvider, IntegrationProvidersError> {
        sqlx::query_as!(
            IntegrationProvider,
            r#"UPDATE integration_providers SET
             name = COALESCE($2, name),
             provider_type = COALESCE($3, provider_type),
             instance_url = COALESCE($4, instance_url),
             api_key = COALESCE($5, api_key)
              WHERE id = $1 AND team_id = $6
              RETURNING id, team_id, name, provider_type as "provider_type!: _", instance_url, api_key, created_at as "created_at!: chrono::DateTime<chrono::Utc>""#,
            params.id,
            params.name,
            params.provider_type as _,
            params.instance_url,
            params.api_key,
            params.team_id,
        )
        .fetch_optional(db)
        .await?
        .ok_or(IntegrationProvidersError::NotFound)
    }

    pub async fn delete<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        team_id: Uuid,
    ) -> Result<(), IntegrationProvidersError> {
        let rows = sqlx::query!(
            "DELETE FROM integration_providers WHERE id = $1 AND team_id = $2",
            id,
            team_id,
        )
        .execute(db)
        .await?
        .rows_affected();

        if rows == 0 {
            return Err(IntegrationProvidersError::NotFound);
        }

        Ok(())
    }
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .map(|db_err| db_err.constraint() == Some("integration_providers_team_name_key"))
        .unwrap_or(false)
}

fn map_sqlx_error(err: sqlx::Error) -> IntegrationProvidersError {
    if is_unique_violation(&err) {
        IntegrationProvidersError::DuplicateName
    } else {
        IntegrationProvidersError::Database(err)
    }
}
