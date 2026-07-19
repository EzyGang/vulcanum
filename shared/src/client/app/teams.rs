use anyhow::Context;
use uuid::Uuid;

use crate::api::app::teams::{AppTeam, AppWorker, UpdateTeamModelsRequest};
use crate::client::{map_response, ApiClient};

impl ApiClient {
    pub async fn list_teams(&self, access_token: &str) -> anyhow::Result<Vec<AppTeam>> {
        let response = self
            .http
            .get(self.app_url("teams"))
            .bearer_auth(access_token)
            .send()
            .await
            .context("list teams request failed")?;
        map_response(response).await.map_err(Into::into)
    }

    pub async fn get_team(&self, id: Uuid, access_token: &str) -> anyhow::Result<AppTeam> {
        let response = self
            .http
            .get(self.app_url(&format!("teams/{id}")))
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

    pub async fn update_team_models(
        &self,
        team_id: Uuid,
        request: &UpdateTeamModelsRequest,
        access_token: &str,
    ) -> anyhow::Result<AppTeam> {
        self.team_patch(&format!("teams/{team_id}"), team_id, request, access_token)
            .await
    }
}
