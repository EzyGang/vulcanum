pub mod queries;

use crate::services::model_providers::errors::ModelProvidersError;

#[derive(Clone, Default)]
pub struct ModelProvidersRepository {}

pub struct CreateOAuthProviderParams<'a> {
    pub display_name: &'a str,
    pub oauth_credentials: &'a serde_json::Value,
    pub oauth_metadata: &'a serde_json::Value,
}

pub struct CreateAuthAttemptParams<'a> {
    pub user_id: &'a str,
    pub device_code: &'a str,
    pub user_code: &'a str,
    pub verification_uri: &'a str,
    pub interval_seconds: i32,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl ModelProvidersRepository {
    pub fn new() -> Self {
        Self {}
    }
}

fn map_sqlx_error(err: sqlx::Error) -> ModelProvidersError {
    let is_duplicate = err
        .as_database_error()
        .map(|db_err| {
            matches!(
                db_err.constraint(),
                Some("model_provider_configs_team_provider_key")
                    | Some("model_provider_configs_team_provider_auth_type")
            )
        })
        .unwrap_or(false);
    if is_duplicate {
        ModelProvidersError::DuplicateProvider
    } else {
        ModelProvidersError::Database(err)
    }
}

fn ensure_rows_affected(rows: u64) -> Result<(), ModelProvidersError> {
    match rows {
        0 => Err(ModelProvidersError::NotFound),
        _ => Ok(()),
    }
}
