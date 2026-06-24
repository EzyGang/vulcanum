use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    ModelProviderConfig, OAuthCredentials, OAuthMetadata, AUTH_TYPE_CHATGPT_OAUTH,
    OPENAI_PROVIDER_KEY,
};
use crate::services::model_providers::service::chatgpt_oauth::{
    extract_account_id, extract_email, oauth_expires_at,
};
use crate::services::model_providers::service::{ModelProvidersService, SelectedModelProviderAuth};

const TOKEN_REFRESH_SKEW_MILLIS: i64 = 60_000;

impl ModelProvidersService {
    pub async fn selected_auth_material(
        &self,
        team_id: Uuid,
        primary_provider_config_id: Option<Uuid>,
        small_provider_config_id: Option<Uuid>,
    ) -> Result<SelectedModelProviderAuth, ModelProvidersError> {
        let ids = selected_ids(primary_provider_config_id, small_provider_config_id);
        if ids.is_empty() {
            return Ok(SelectedModelProviderAuth::default());
        }
        let mut providers = self.repo.find_by_ids(&self.db, team_id, &ids).await?;
        if providers.len() != ids.len() {
            return Err(ModelProvidersError::NotFound);
        }
        validate_auth_compatibility(&providers)?;
        self.refresh_expired_chatgpt_oauth(&mut providers).await?;
        let opencode_auth_content = self.opencode_auth_content(&providers)?;
        Ok(SelectedModelProviderAuth {
            providers,
            opencode_auth_content,
        })
    }

    async fn refresh_expired_chatgpt_oauth(
        &self,
        providers: &mut [ModelProviderConfig],
    ) -> Result<(), ModelProvidersError> {
        for provider in providers {
            if provider.auth_type != AUTH_TYPE_CHATGPT_OAUTH {
                continue;
            }
            let Some(encrypted) = provider.oauth_credentials.as_ref() else {
                return Err(ModelProvidersError::InvalidSelection(
                    "ChatGPT OAuth provider is missing credentials".to_owned(),
                ));
            };
            let credentials: OAuthCredentials = self.cipher.decrypt_json(encrypted)?;
            if !credentials_need_refresh(credentials.expires) {
                continue;
            }

            let refreshed = self
                .oauth_client
                .refresh_access_token(&credentials.refresh)
                .await?;
            let expires_at = oauth_expires_at(refreshed.expires_in);
            let (stored_account_id, stored_email) = stored_oauth_metadata(&provider.oauth_metadata);
            let account_id = refreshed
                .id_token
                .as_deref()
                .and_then(extract_account_id)
                .or_else(|| extract_account_id(&refreshed.access_token))
                .or(stored_account_id);
            let email = refreshed
                .id_token
                .as_deref()
                .and_then(extract_email)
                .or(stored_email);
            let next_credentials = OAuthCredentials {
                access: refreshed.access_token,
                refresh: refreshed.refresh_token.unwrap_or(credentials.refresh),
                expires: expires_at.timestamp_millis(),
                account_id: account_id.clone(),
            };
            let next_metadata = OAuthMetadata {
                account_id,
                email,
                expires_at: Some(expires_at),
            };
            let encrypted = self.cipher.encrypt_json(&next_credentials)?;
            let metadata_json = serde_json::to_value(next_metadata)
                .map_err(|_| ModelProvidersError::Serialization)?;
            *provider = self
                .repo
                .update_chatgpt_oauth_credentials(
                    &self.db,
                    provider.id,
                    provider.team_id,
                    &encrypted,
                    &metadata_json,
                )
                .await?;
        }
        Ok(())
    }

    fn opencode_auth_content(
        &self,
        providers: &[ModelProviderConfig],
    ) -> Result<Option<String>, ModelProvidersError> {
        let Some(provider) = providers
            .iter()
            .find(|provider| provider.auth_type == AUTH_TYPE_CHATGPT_OAUTH)
        else {
            return Ok(None);
        };
        let Some(encrypted) = provider.oauth_credentials.as_ref() else {
            return Err(ModelProvidersError::InvalidSelection(
                "ChatGPT OAuth provider is missing credentials".to_owned(),
            ));
        };
        let credentials: OAuthCredentials = self.cipher.decrypt_json(encrypted)?;
        let mut openai = serde_json::Map::new();
        openai.insert("type".to_owned(), json!("oauth"));
        openai.insert("refresh".to_owned(), json!(credentials.refresh));
        openai.insert("access".to_owned(), json!(credentials.access));
        openai.insert("expires".to_owned(), json!(credentials.expires));
        if let Some(account_id) = credentials.account_id {
            openai.insert("accountId".to_owned(), json!(account_id));
        }
        Ok(Some(json!({ OPENAI_PROVIDER_KEY: openai }).to_string()))
    }
}

fn credentials_need_refresh(expires: i64) -> bool {
    expires <= Utc::now().timestamp_millis() + TOKEN_REFRESH_SKEW_MILLIS
}

fn stored_oauth_metadata(metadata: &serde_json::Value) -> (Option<String>, Option<String>) {
    match serde_json::from_value::<OAuthMetadata>(metadata.clone()).ok() {
        Some(metadata) => (metadata.account_id, metadata.email),
        None => (None, None),
    }
}

fn selected_ids(primary: Option<Uuid>, small: Option<Uuid>) -> Vec<Uuid> {
    let mut ids = Vec::new();
    for id in [primary, small].into_iter().flatten() {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

fn validate_auth_compatibility(
    providers: &[ModelProviderConfig],
) -> Result<(), ModelProvidersError> {
    for (index, provider) in providers.iter().enumerate() {
        for other in providers.iter().skip(index + 1) {
            if provider.provider_key == other.provider_key && provider.auth_type != other.auth_type
            {
                return Err(ModelProvidersError::InvalidSelection(format!(
                    "{} cannot use multiple auth modes in one job",
                    provider.provider_key
                )));
            }
        }
    }
    Ok(())
}
