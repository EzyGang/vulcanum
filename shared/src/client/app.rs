use anyhow::Context;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::api::app::{AppModelProvider, AppTeam, AppWorker, GithubAppInstallation, TaskTracker};
use crate::api::wire::{AuthTokenResponse, RefreshRequest};
use crate::client::{map_response, ApiClient};

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

    pub async fn list_teams(&self, access_token: &str) -> anyhow::Result<Vec<AppTeam>> {
        let url = format!("{}/api/v1/teams", self.base_url);
        let response = self
            .http
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .context("list teams request failed")?;

        map_response(response).await.map_err(Into::into)
    }

    pub async fn get_team(&self, id: Uuid, access_token: &str) -> anyhow::Result<AppTeam> {
        let url = format!("{}/api/v1/teams/{id}", self.base_url);
        let response = self
            .http
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .context("get team request failed")?;

        map_response(response).await.map_err(Into::into)
    }

    pub async fn list_workers(
        &self,
        team_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<Vec<AppWorker>> {
        self.team_get("workers", team_id, access_token).await
    }

    pub async fn list_task_trackers(
        &self,
        team_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<Vec<TaskTracker>> {
        self.team_get("providers", team_id, access_token).await
    }

    pub async fn list_model_providers(
        &self,
        team_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<Vec<AppModelProvider>> {
        self.team_get("model-providers", team_id, access_token)
            .await
    }

    pub async fn get_github_app_installation(
        &self,
        team_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<Option<GithubAppInstallation>> {
        self.team_get("github/installation", team_id, access_token)
            .await
    }

    async fn team_get<T>(&self, path: &str, team_id: Uuid, access_token: &str) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}/api/v1/{path}", self.base_url);
        let response = self
            .http
            .get(&url)
            .bearer_auth(access_token)
            .header("X-Team-Id", team_id.to_string())
            .send()
            .await
            .with_context(|| format!("{path} request failed"))?;

        map_response(response).await.map_err(Into::into)
    }
}
