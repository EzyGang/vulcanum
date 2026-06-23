use serde_json::json;
use uuid::Uuid;

use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    ModelProviderConfig, OAuthCredentials, AUTH_TYPE_CHATGPT_OAUTH, OPENAI_PROVIDER_KEY,
};
use crate::services::model_providers::service::{ModelProvidersService, SelectedModelProviderAuth};

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
        let providers = self.repo.find_by_ids(&self.db, team_id, &ids).await?;
        if providers.len() != ids.len() {
            return Err(ModelProvidersError::NotFound);
        }
        validate_auth_compatibility(&providers)?;
        let opencode_auth_content = self.opencode_auth_content(&providers)?;
        Ok(SelectedModelProviderAuth {
            providers,
            opencode_auth_content,
        })
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
