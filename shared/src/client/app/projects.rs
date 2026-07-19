use uuid::Uuid;

use crate::api::app::projects::{
    AppProject, CreateProjectRequest, ProviderProject, ProviderWorkspace, UpdateProjectRequest,
};
use crate::client::ApiClient;

impl ApiClient {
    pub async fn list_projects(
        &self,
        team_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<Vec<AppProject>> {
        self.team_get("projects", team_id, access_token).await
    }

    pub async fn get_project(
        &self,
        team_id: Uuid,
        id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<AppProject> {
        self.team_get(&format!("projects/{id}"), team_id, access_token)
            .await
    }

    pub async fn create_project(
        &self,
        team_id: Uuid,
        request: &CreateProjectRequest,
        access_token: &str,
    ) -> anyhow::Result<AppProject> {
        self.team_post("projects", team_id, request, access_token)
            .await
    }

    pub async fn update_project(
        &self,
        team_id: Uuid,
        id: Uuid,
        request: &UpdateProjectRequest,
        access_token: &str,
    ) -> anyhow::Result<AppProject> {
        self.team_patch(&format!("projects/{id}"), team_id, request, access_token)
            .await
    }

    pub async fn list_provider_workspaces(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<Vec<ProviderWorkspace>> {
        self.team_get(
            &format!("providers/{provider_id}/workspaces"),
            team_id,
            access_token,
        )
        .await
    }

    pub async fn list_provider_projects(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        workspace_id: &str,
        access_token: &str,
    ) -> anyhow::Result<Vec<ProviderProject>> {
        let path = format!("providers/{provider_id}/projects");
        let request = self
            .http
            .get(self.app_url(&path))
            .query(&[("workspace_id", workspace_id)]);
        self.send_team_json(request, &path, team_id, access_token)
            .await
    }
}
