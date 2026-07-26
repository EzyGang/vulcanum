use super::*;

impl ModelProvidersService {
    pub async fn catalog(&self) -> Result<CatalogResponse, ModelProvidersError> {
        self.catalog.catalog().await
    }

    pub async fn list_all(
        &self,
        team_id: Uuid,
    ) -> Result<Vec<ModelProviderResponse>, ModelProvidersError> {
        let providers = self.repo.list_all(&self.db, team_id).await?;
        let mut responses = Vec::with_capacity(providers.len());
        for provider in providers {
            let response = to_response(provider, &self.cipher)?;
            self.catalog
                .validate_credential_fields(&response.provider_key, &response.credential_fields)
                .await?;
            responses.push(response);
        }
        Ok(responses)
    }

    pub async fn render_agent_config_for_team(
        &self,
        team_id: Uuid,
        backend: AgentBackend,
        selection: ModelSelection<'_>,
    ) -> Result<RenderedAgentConfig, ModelProvidersError> {
        let selected_keys = selected_provider_keys(&selection);
        let mut providers = self.repo.list_all(&self.db, team_id).await?;
        if !selected_keys.is_empty() {
            providers.retain(|provider| selected_keys.contains(provider.provider_key.as_str()));
        }
        for provider in &mut providers {
            self.validate_stored_api_key_credentials(provider).await?;
            self.refresh_provider_if_needed(provider).await?;
        }
        render_agent_config(backend, &providers, &self.cipher, selection)
    }
}
