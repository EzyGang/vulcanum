use uuid::Uuid;

use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    ChatGptAuthStatusResponse, AUTH_TYPE_CHATGPT_OAUTH, OPENAI_PROVIDER_KEY,
};
use crate::services::model_providers::service::chatgpt_oauth::CHATGPT_AUTH_COMPLETE;
use crate::services::model_providers::service::ModelProvidersService;

impl ModelProvidersService {
    pub(super) async fn auth_status_from_current_attempt(
        &self,
        team_id: Uuid,
        user_id: &str,
        attempt_id: Uuid,
    ) -> Result<ChatGptAuthStatusResponse, ModelProvidersError> {
        let attempt = self
            .repo
            .find_auth_attempt(&self.db, attempt_id, team_id, user_id)
            .await?;
        self.auth_status_from_attempt(team_id, &attempt.status, attempt.error)
            .await
    }

    pub(super) async fn auth_status_from_attempt(
        &self,
        team_id: Uuid,
        status: &str,
        error: Option<String>,
    ) -> Result<ChatGptAuthStatusResponse, ModelProvidersError> {
        let provider = match status {
            CHATGPT_AUTH_COMPLETE => self
                .repo
                .find_by_provider_auth(
                    &self.db,
                    team_id,
                    OPENAI_PROVIDER_KEY,
                    AUTH_TYPE_CHATGPT_OAUTH,
                )
                .await
                .ok(),
            _ => None,
        };
        Ok(ChatGptAuthStatusResponse {
            status: status.to_owned(),
            error,
            poll_interval_seconds: None,
            provider,
        })
    }
}
