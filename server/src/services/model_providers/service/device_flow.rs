use super::{
    encrypted_oauth_credentials, to_response, DevicePoll, ModelProvidersError,
    ModelProvidersService, PendingDeviceFlow, PollDeviceFlowResponse, StartDeviceFlowRequest,
    StartDeviceFlowResponse, Utc, Uuid,
};

impl ModelProvidersService {
    pub async fn start_device_flow(
        &self,
        team_id: Uuid,
        user_id: Option<&str>,
        params: StartDeviceFlowRequest,
    ) -> Result<StartDeviceFlowResponse, ModelProvidersError> {
        if params.provider_key != self.device_auth_provider.model_provider_key()
            || params.device_provider != self.device_auth_provider.provider_id()
        {
            return Err(ModelProvidersError::InvalidAuthConfig(
                "unsupported device flow provider".to_owned(),
            ));
        }
        let device_start = self.device_auth_provider.start().await?;
        let now = Utc::now();
        let expires_at = now + chrono::Duration::minutes(10);
        let next_poll_at = now + chrono::Duration::seconds(device_start.interval_seconds);
        let attempt_id = Uuid::new_v4();

        self.device_flow_store
            .insert(PendingDeviceFlow {
                attempt_id,
                team_id,
                user_id: user_id.map(str::to_owned),
                provider_key: params.provider_key,
                device_provider: params.device_provider,
                display_name: match params.display_name.is_empty() {
                    true => "ChatGPT Plus".to_owned(),
                    false => params.display_name,
                },
                device_auth_id: device_start.device_auth_id,
                user_code: device_start.user_code.clone(),
                verification_uri: device_start.verification_uri.clone(),
                interval_seconds: device_start.interval_seconds,
                next_poll_at,
                expires_at,
            })
            .await?;

        Ok(StartDeviceFlowResponse {
            attempt_id,
            verification_uri: device_start.verification_uri,
            user_code: device_start.user_code,
            interval_seconds: device_start.interval_seconds,
            expires_at,
        })
    }

    pub async fn poll_device_flow(
        &self,
        team_id: Uuid,
        user_id: Option<&str>,
        attempt_id: Uuid,
    ) -> Result<PollDeviceFlowResponse, ModelProvidersError> {
        let Some(pending) = self.device_flow_store.get(attempt_id).await? else {
            return Err(ModelProvidersError::DeviceFlowExpired);
        };
        if pending.team_id != team_id || pending.user_id.as_deref() != user_id {
            return Err(ModelProvidersError::DeviceFlowExpired);
        }
        let now = Utc::now();
        if pending.expires_at <= now {
            self.device_flow_store.consume(attempt_id).await?;
            return Err(ModelProvidersError::DeviceFlowExpired);
        }
        if pending.next_poll_at > now {
            return Ok(PollDeviceFlowResponse::Pending {
                next_poll_at: pending.next_poll_at,
            });
        }

        match self.device_auth_provider.poll(&pending).await? {
            DevicePoll::Pending => {
                let next_poll_at = now + chrono::Duration::seconds(pending.interval_seconds);
                self.device_flow_store
                    .update_next_poll(attempt_id, next_poll_at)
                    .await?;
                Ok(PollDeviceFlowResponse::Pending { next_poll_at })
            }
            DevicePoll::Complete(credential) => {
                self.device_flow_store.consume(attempt_id).await?;
                let credentials = encrypted_oauth_credentials(&credential, &self.cipher)?;
                let provider = self
                    .repo
                    .upsert_by_provider_key(
                        &self.db,
                        team_id,
                        &pending.provider_key,
                        &pending.display_name,
                        &credentials,
                    )
                    .await?;
                let response = to_response(provider, &self.cipher)?;
                self.catalog
                    .validate_credential_fields(&response.provider_key, &response.credential_fields)
                    .await?;
                Ok(PollDeviceFlowResponse::Connected {
                    provider: Box::new(response),
                })
            }
        }
    }
}
