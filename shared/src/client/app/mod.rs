mod github;
mod model_providers;
mod projects;
mod task_trackers;
mod teams;
mod work_runs;

use anyhow::Context;
use reqwest::RequestBuilder;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use crate::api::wire::{AuthTokenResponse, RefreshRequest};
use crate::client::{build_error, map_response, ApiClient};

impl ApiClient {
    pub async fn refresh_app_session(
        &self,
        refresh_token: &str,
    ) -> anyhow::Result<AuthTokenResponse> {
        let url = format!("{}/api/v1/auth/refresh", self.base_url);
        let body = RefreshRequest {
            refresh_token: refresh_token.to_owned(),
        };
        let response = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("app session refresh request failed")?;

        map_response(response).await.map_err(Into::into)
    }

    async fn team_get<T>(&self, path: &str, team_id: Uuid, access_token: &str) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        let request = self.http.get(self.app_url(path));
        self.send_team_json(request, path, team_id, access_token)
            .await
    }

    async fn team_post<B, T>(
        &self,
        path: &str,
        team_id: Uuid,
        body: &B,
        access_token: &str,
    ) -> anyhow::Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let request = self.http.post(self.app_url(path)).json(body);
        self.send_team_json(request, path, team_id, access_token)
            .await
    }

    async fn team_patch<B, T>(
        &self,
        path: &str,
        team_id: Uuid,
        body: &B,
        access_token: &str,
    ) -> anyhow::Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let request = self.http.patch(self.app_url(path)).json(body);
        self.send_team_json(request, path, team_id, access_token)
            .await
    }

    async fn team_delete(
        &self,
        path: &str,
        team_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<()> {
        let response = self
            .scoped(self.http.delete(self.app_url(path)), team_id, access_token)
            .send()
            .await
            .with_context(|| format!("{path} request failed"))?;
        if response.status().is_success() {
            return Ok(());
        }

        Err(build_error(response).await.into())
    }

    async fn send_team_json<T>(
        &self,
        request: RequestBuilder,
        path: &str,
        team_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .scoped(request, team_id, access_token)
            .send()
            .await
            .with_context(|| format!("{path} request failed"))?;
        map_response(response).await.map_err(Into::into)
    }

    fn scoped(&self, request: RequestBuilder, team_id: Uuid, access_token: &str) -> RequestBuilder {
        request
            .bearer_auth(access_token)
            .header("X-Team-Id", team_id.to_string())
    }

    fn app_url(&self, path: &str) -> String {
        format!("{}/api/v1/{path}", self.base_url)
    }
}
