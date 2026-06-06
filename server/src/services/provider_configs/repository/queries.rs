use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::provider_configs::errors::IntegrationProvidersError;
use crate::services::provider_configs::model::{CreateProviderRequest, IntegrationProvider};
use crate::services::provider_configs::repository::IntegrationProvidersRepository;
use crate::services::providers::model::IntegrationType;

impl IntegrationProvidersRepository {
    pub async fn list_all<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<Vec<IntegrationProvider>, IntegrationProvidersError> {
        sqlx::query_as!(
            IntegrationProvider,
            r#"SELECT id, name, provider_type as "provider_type!: _", instance_url, api_key, created_at as "created_at!: DateTime<Utc>"
             FROM integration_providers ORDER BY created_at DESC"#,
        )
        .fetch_all(db)
        .await
        .map_err(IntegrationProvidersError::from)
    }

    pub async fn find_by_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<IntegrationProvider, IntegrationProvidersError> {
        sqlx::query_as!(
            IntegrationProvider,
            r#"SELECT id, name, provider_type as "provider_type!: _", instance_url, api_key, created_at as "created_at!: DateTime<Utc>"
             FROM integration_providers WHERE id = $1"#,
            id,
        )
        .fetch_optional(db)
        .await?
        .ok_or(IntegrationProvidersError::NotFound)
    }

    pub async fn create<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        params: &CreateProviderRequest,
    ) -> Result<IntegrationProvider, IntegrationProvidersError> {
        let id = Uuid::new_v4();
        let provider_type = params.provider_type.unwrap_or_default();

        sqlx::query_as!(
            IntegrationProvider,
            r#"INSERT INTO integration_providers (id, name, provider_type, instance_url, api_key)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, name, provider_type as "provider_type!: _", instance_url, api_key, created_at as "created_at!: DateTime<Utc>""#,
            id,
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
        id: Uuid,
        name: Option<&str>,
        provider_type: Option<IntegrationType>,
        instance_url: Option<&str>,
        api_key: Option<&str>,
    ) -> Result<IntegrationProvider, IntegrationProvidersError> {
        sqlx::query_as!(
            IntegrationProvider,
            r#"UPDATE integration_providers SET
             name = COALESCE($2, name),
             provider_type = COALESCE($3, provider_type),
             instance_url = COALESCE($4, instance_url),
             api_key = COALESCE($5, api_key)
             WHERE id = $1
             RETURNING id, name, provider_type as "provider_type!: _", instance_url, api_key, created_at as "created_at!: DateTime<Utc>""#,
            id,
            name,
            provider_type as _,
            instance_url,
            api_key,
        )
        .fetch_optional(db)
        .await?
        .ok_or(IntegrationProvidersError::NotFound)
    }

    pub async fn delete<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<(), IntegrationProvidersError> {
        let rows = sqlx::query!("DELETE FROM integration_providers WHERE id = $1", id)
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
        .map(|db_err| db_err.constraint() == Some("integration_providers_name_key"))
        .unwrap_or(false)
}

fn map_sqlx_error(err: sqlx::Error) -> IntegrationProvidersError {
    if is_unique_violation(&err) {
        IntegrationProvidersError::DuplicateName
    } else {
        IntegrationProvidersError::Database(err)
    }
}
