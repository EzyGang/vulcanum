use super::*;

impl ModelProvidersService {
    pub async fn create(
        &self,
        team_id: Uuid,
        params: CreateModelProviderRequest,
    ) -> Result<ModelProviderResponse, ModelProvidersError> {
        self.catalog.validate_provider(&params.provider_key).await?;
        let mut stored = params.clone();
        match params.auth_type {
            ModelProviderAuthType::ApiKey => {
                self.validate_api_key_credentials(&params.provider_key, &params.credentials)
                    .await?;
                stored.credentials =
                    encrypted_api_key_credentials(&params.credentials, &self.cipher)?;
            }
            ModelProviderAuthType::None => {
                reject_credentials_for_none_auth(&params.credentials)?;
                stored.credentials = serde_json::Value::Null;
            }
            ModelProviderAuthType::DeviceOauth => {
                return Err(ModelProvidersError::InvalidAuthConfig(
                    "device OAuth must be connected with the device flow endpoint".to_owned(),
                ));
            }
        }
        let provider = self.repo.create(&self.db, team_id, &stored).await?;
        to_response(provider, &self.cipher)
    }

    pub async fn update(
        &self,
        id: Uuid,
        team_id: Uuid,
        params: UpdateModelProviderRequest,
    ) -> Result<ModelProviderResponse, ModelProvidersError> {
        let mut stored = params.clone();
        match params.auth_type {
            Some(ModelProviderAuthType::DeviceOauth) => {
                return Err(ModelProvidersError::InvalidAuthConfig(
                    "device OAuth must be connected with the device flow endpoint".to_owned(),
                ));
            }
            Some(ModelProviderAuthType::None) => {
                reject_optional_credentials_for_none_auth(params.credentials.as_ref())?;
                stored.credentials = Some(serde_json::Value::Null);
            }
            Some(ModelProviderAuthType::ApiKey) => {
                let credentials = params.credentials.as_ref().ok_or_else(|| {
                    ModelProvidersError::InvalidAuthConfig(
                        "api key auth requires credentials".to_owned(),
                    )
                })?;
                let existing = self.repo.find_by_id(&self.db, id, team_id).await?;
                self.validate_api_key_credentials(&existing.provider_key, credentials)
                    .await?;
                stored.credentials =
                    Some(encrypted_api_key_credentials(credentials, &self.cipher)?);
            }
            None => {
                if let Some(credentials) = params.credentials.as_ref() {
                    let existing = self.repo.find_by_id(&self.db, id, team_id).await?;
                    self.validate_api_key_credentials(&existing.provider_key, credentials)
                        .await?;
                    stored.credentials =
                        Some(encrypted_api_key_credentials(credentials, &self.cipher)?);
                }
            }
        }
        let provider = self.repo.update(&self.db, id, team_id, &stored).await?;
        to_response(provider, &self.cipher)
    }
    pub async fn delete(&self, id: Uuid, team_id: Uuid) -> Result<(), ModelProvidersError> {
        self.repo.delete(&self.db, id, team_id).await
    }
}
