use anyhow::Context;

use crate::api::wire::{
    AuthExchangeRequest, AuthModeResponse, AuthTokenResponse, InstanceLoginRequest,
};
use crate::client::{map_response, ApiClient};

impl ApiClient {
    pub async fn auth_mode(&self) -> anyhow::Result<AuthModeResponse> {
        let url = format!("{}/api/v1/auth/mode", self.base_url);
        let response = self
            .http
            .get(&url)
            .send()
            .await
            .context("auth mode request failed")?;

        map_response(response).await.map_err(Into::into)
    }

    pub async fn instance_login(&self, password: &str) -> anyhow::Result<AuthTokenResponse> {
        let url = format!("{}/api/v1/auth/instance-login", self.base_url);
        let body = InstanceLoginRequest {
            password: password.to_owned(),
        };
        let response = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("instance login request failed")?;

        map_response(response).await.map_err(Into::into)
    }

    pub async fn exchange_auth_code(&self, code: &str) -> anyhow::Result<AuthTokenResponse> {
        let url = format!("{}/api/v1/auth/exchange", self.base_url);
        let body = AuthExchangeRequest {
            code: code.to_owned(),
        };
        let response = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("auth code exchange request failed")?;

        map_response(response).await.map_err(Into::into)
    }
}
