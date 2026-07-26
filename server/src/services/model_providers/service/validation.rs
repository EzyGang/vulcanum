use super::{
    api_key_credential_fields, encrypted_oauth_credentials, is_codex_compatible_openai_model,
    parse_auth, ModelProviderConfig, ModelProvidersError, ModelProvidersService, ParsedAuth, Utc,
    Uuid, OPENAI_PROVIDER_KEY,
};

impl ModelProvidersService {
    pub async fn validate_model_selection(
        &self,
        team_id: Uuid,
        provider_key: Option<&str>,
        model_id: Option<&str>,
    ) -> Result<(), ModelProvidersError> {
        let Some(provider_key) = provider_key.filter(|value| !value.is_empty()) else {
            return Ok(());
        };
        let Some(model_id) = model_id.filter(|value| !value.is_empty()) else {
            return Ok(());
        };

        let provider = self
            .repo
            .find_by_provider_key(&self.db, team_id, provider_key)
            .await?;
        match parse_auth(&provider.credentials, &self.cipher)? {
            ParsedAuth::DeviceOAuth(_) if provider_key == OPENAI_PROVIDER_KEY => {
                return match is_codex_compatible_openai_model(model_id) {
                    true => Ok(()),
                    false => Err(ModelProvidersError::UnknownModel {
                        provider_key: provider_key.to_owned(),
                        model_id: model_id.to_owned(),
                    }),
                };
            }
            _ => (),
        }
        self.catalog.validate_model(provider_key, model_id).await
    }

    pub(super) async fn refresh_provider_if_needed(
        &self,
        provider: &mut ModelProviderConfig,
    ) -> Result<(), ModelProvidersError> {
        let ParsedAuth::DeviceOAuth(credential) = parse_auth(&provider.credentials, &self.cipher)?
        else {
            return Ok(());
        };
        if provider.provider_key != self.device_auth_provider.model_provider_key()
            || credential.provider != self.device_auth_provider.provider_id()
        {
            return Err(ModelProvidersError::InvalidAuthConfig(format!(
                "unsupported device OAuth provider {}",
                credential.provider
            )));
        }
        if !credential.should_refresh(Utc::now()) {
            return Ok(());
        }

        let refreshed = self.device_auth_provider.refresh(&credential).await?;
        let credentials = encrypted_oauth_credentials(&refreshed, &self.cipher)?;
        let updated = self
            .repo
            .update_credentials(&self.db, provider.id, provider.team_id, &credentials)
            .await?;
        *provider = updated;
        Ok(())
    }

    pub(super) async fn validate_api_key_credentials(
        &self,
        provider_key: &str,
        credentials: &serde_json::Value,
    ) -> Result<(), ModelProvidersError> {
        let fields = api_key_credential_fields(credentials)?;
        if fields.is_empty() {
            return Err(ModelProvidersError::InvalidAuthConfig(
                "api key auth requires at least one credential".to_owned(),
            ));
        }
        self.catalog
            .validate_credential_fields(provider_key, &fields)
            .await
    }

    pub(super) async fn validate_stored_api_key_credentials(
        &self,
        provider: &ModelProviderConfig,
    ) -> Result<(), ModelProvidersError> {
        let ParsedAuth::ApiKey(credentials) = parse_auth(&provider.credentials, &self.cipher)?
        else {
            return Ok(());
        };
        let mut fields = credentials.keys().cloned().collect::<Vec<String>>();
        fields.sort();
        self.catalog
            .validate_credential_fields(&provider.provider_key, &fields)
            .await
    }
}
