use uuid::Uuid;

use crate::api::app::github::{GithubAppInstallation, GithubAuthUrlResponse};
use crate::client::ApiClient;

impl ApiClient {
    pub async fn get_github_auth_url(
        &self,
        team_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<GithubAuthUrlResponse> {
        self.team_get("github/auth-url", team_id, access_token)
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

    pub async fn delete_github_app_installation(
        &self,
        team_id: Uuid,
        installation_id: i64,
        access_token: &str,
    ) -> anyhow::Result<()> {
        self.team_delete(
            &format!("github/installation/{installation_id}"),
            team_id,
            access_token,
        )
        .await
    }
}
