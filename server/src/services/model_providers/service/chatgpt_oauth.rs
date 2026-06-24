use chrono::Utc;
use uuid::Uuid;

use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    ChatGptAuthStartResponse, ChatGptAuthStatusResponse, OAuthCredentials, OAuthMetadata,
    StartChatGptAuthRequest,
};
use crate::services::model_providers::repository::{
    CreateAuthAttemptParams, CreateOAuthProviderParams,
};
use crate::services::model_providers::service::oauth_client::DevicePollOutcome;
use crate::services::model_providers::service::oauth_tokens::{
    extract_account_id, extract_email, oauth_expires_at,
};
use crate::services::model_providers::service::ModelProvidersService;

const CHATGPT_AUTH_PENDING: &str = "pending";
pub(super) const CHATGPT_AUTH_COMPLETE: &str = "complete";
const CHATGPT_AUTH_EXPIRED: &str = "expired";
const CHATGPT_AUTH_FAILED: &str = "failed";
const DEFAULT_CHATGPT_DISPLAY_NAME: &str = "OpenAI ChatGPT Pro/Plus";
const DEFAULT_DEVICE_POLL_SECONDS: i32 = 5;
const DEVICE_POLL_SLOW_DOWN_SECONDS: i32 = 5;

impl ModelProvidersService {
    pub async fn start_chatgpt_auth(
        &self,
        team_id: Uuid,
        user_id: &str,
        params: StartChatGptAuthRequest,
    ) -> Result<ChatGptAuthStartResponse, ModelProvidersError> {
        let body = self.oauth_client.start_device_flow().await?;
        let encrypted_device_code = self.cipher.encrypt_json(&body.device_code)?;
        let display_name = display_name_or_default(&params.display_name);
        let mut tx = self.db.begin().await?;
        self.repo
            .fail_pending_auth_attempts_for_user(
                &mut *tx,
                team_id,
                user_id,
                "Superseded by a newer device login",
            )
            .await?;
        let attempt = self
            .repo
            .create_auth_attempt(
                &mut *tx,
                team_id,
                &CreateAuthAttemptParams {
                    user_id,
                    encrypted_device_code: &encrypted_device_code,
                    user_code: &body.user_code,
                    verification_uri: &body.verification_uri,
                    display_name: &display_name,
                    interval_seconds: body.interval.unwrap_or(DEFAULT_DEVICE_POLL_SECONDS),
                    expires_at: Utc::now() + chrono::Duration::seconds(body.expires_in),
                },
            )
            .await?;
        tx.commit().await?;

        Ok(ChatGptAuthStartResponse {
            attempt_id: attempt.id,
            verification_uri: attempt.verification_uri,
            user_code: attempt.user_code,
            expires_at: attempt.expires_at,
            poll_interval_seconds: attempt.interval_seconds,
        })
    }

    pub async fn chatgpt_auth_status(
        &self,
        team_id: Uuid,
        user_id: &str,
        attempt_id: Uuid,
    ) -> Result<ChatGptAuthStatusResponse, ModelProvidersError> {
        let attempt = self
            .repo
            .find_auth_attempt(&self.db, attempt_id, team_id, user_id)
            .await?;
        if attempt.status != CHATGPT_AUTH_PENDING {
            return self
                .auth_status_from_attempt(team_id, &attempt.status, attempt.error)
                .await;
        }
        if attempt.expires_at <= Utc::now() {
            let updated = self
                .repo
                .try_update_auth_attempt_status_from(
                    &self.db,
                    attempt.id,
                    CHATGPT_AUTH_PENDING,
                    CHATGPT_AUTH_EXPIRED,
                    Some("Device login expired"),
                )
                .await?;
            if !updated {
                return self
                    .auth_status_from_current_attempt(team_id, user_id, attempt.id)
                    .await;
            }
            return Ok(ChatGptAuthStatusResponse {
                status: CHATGPT_AUTH_EXPIRED.to_owned(),
                error: Some("Device login expired".to_owned()),
                poll_interval_seconds: None,
                provider: None,
            });
        }

        let device_code: String = self.cipher.decrypt_json(&attempt.encrypted_device_code)?;
        let code = match self.oauth_client.poll_device_token(&device_code).await? {
            DevicePollOutcome::Pending => {
                return Ok(ChatGptAuthStatusResponse {
                    status: CHATGPT_AUTH_PENDING.to_owned(),
                    error: None,
                    poll_interval_seconds: Some(attempt.interval_seconds),
                    provider: None,
                });
            }
            DevicePollOutcome::SlowDown => {
                let interval_seconds = attempt.interval_seconds + DEVICE_POLL_SLOW_DOWN_SECONDS;
                let updated = self
                    .repo
                    .try_update_auth_attempt_interval_from(
                        &self.db,
                        attempt.id,
                        CHATGPT_AUTH_PENDING,
                        interval_seconds,
                    )
                    .await?;
                if !updated {
                    return self
                        .auth_status_from_current_attempt(team_id, user_id, attempt.id)
                        .await;
                }
                return Ok(ChatGptAuthStatusResponse {
                    status: CHATGPT_AUTH_PENDING.to_owned(),
                    error: None,
                    poll_interval_seconds: Some(interval_seconds),
                    provider: None,
                });
            }
            DevicePollOutcome::Failed(message) => {
                let updated = self
                    .repo
                    .try_update_auth_attempt_status_from(
                        &self.db,
                        attempt.id,
                        CHATGPT_AUTH_PENDING,
                        CHATGPT_AUTH_FAILED,
                        Some(&message),
                    )
                    .await?;
                if !updated {
                    return self
                        .auth_status_from_current_attempt(team_id, user_id, attempt.id)
                        .await;
                }
                return Ok(ChatGptAuthStatusResponse {
                    status: CHATGPT_AUTH_FAILED.to_owned(),
                    error: Some(message),
                    poll_interval_seconds: None,
                    provider: None,
                });
            }
            DevicePollOutcome::Authorized(code) => code,
        };

        let token = self.oauth_client.exchange_authorization_code(&code).await?;
        let expires_at = oauth_expires_at(token.expires_in);
        let account_id = token
            .id_token
            .as_deref()
            .and_then(extract_account_id)
            .or_else(|| extract_account_id(&token.access_token));
        let email = token.id_token.as_deref().and_then(extract_email);
        let credentials = OAuthCredentials {
            access: token.access_token,
            refresh: token.refresh_token,
            expires: expires_at.timestamp_millis(),
            account_id: account_id.clone(),
        };
        let metadata = OAuthMetadata {
            account_id,
            email,
            expires_at: Some(expires_at),
        };
        let encrypted = self.cipher.encrypt_json(&credentials)?;
        let metadata_json =
            serde_json::to_value(metadata).map_err(|_| ModelProvidersError::Serialization)?;

        let mut tx = self.db.begin().await?;
        let updated = self
            .repo
            .try_update_auth_attempt_status_from(
                &mut *tx,
                attempt.id,
                CHATGPT_AUTH_PENDING,
                CHATGPT_AUTH_COMPLETE,
                None,
            )
            .await?;
        if !updated {
            tx.rollback().await?;
            return self
                .auth_status_from_current_attempt(team_id, user_id, attempt.id)
                .await;
        }
        let provider = self
            .repo
            .upsert_chatgpt_oauth(
                &mut *tx,
                team_id,
                &CreateOAuthProviderParams {
                    display_name: &attempt.display_name,
                    oauth_credentials: &encrypted,
                    oauth_metadata: &metadata_json,
                },
            )
            .await?;
        tx.commit().await?;

        Ok(ChatGptAuthStatusResponse {
            status: CHATGPT_AUTH_COMPLETE.to_owned(),
            error: None,
            poll_interval_seconds: None,
            provider: Some(provider),
        })
    }

    pub async fn cancel_chatgpt_auth(
        &self,
        team_id: Uuid,
        user_id: &str,
        attempt_id: Uuid,
    ) -> Result<(), ModelProvidersError> {
        let attempt = self
            .repo
            .find_auth_attempt(&self.db, attempt_id, team_id, user_id)
            .await?;
        if attempt.status != CHATGPT_AUTH_PENDING {
            return Ok(());
        }
        self.repo
            .try_update_auth_attempt_status_from(
                &self.db,
                attempt.id,
                CHATGPT_AUTH_PENDING,
                CHATGPT_AUTH_FAILED,
                Some("Device login cancelled"),
            )
            .await?;
        Ok(())
    }
}

fn display_name_or_default(display_name: &str) -> String {
    match display_name.trim() {
        "" => DEFAULT_CHATGPT_DISPLAY_NAME.to_owned(),
        value => value.to_owned(),
    }
}
