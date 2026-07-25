use uuid::Uuid;

use crate::models::github_app::errors::GithubAppError;
use crate::models::github_app::model::GithubInstallation;
use crate::services::github_app::service::GithubAppManager;

impl GithubAppManager {
    pub async fn connect_single_installation(
        &self,
        team_id: Uuid,
        installed_by_user_id: Option<&str>,
    ) -> Result<GithubInstallation, GithubAppError> {
        match self.repo.get_installation(&self.db, team_id).await? {
            Some(existing) => {
                self.repo
                    .insert_installation(
                        &self.db,
                        team_id,
                        installed_by_user_id,
                        existing.github_installation_id,
                        &existing.account_login,
                    )
                    .await
            }
            None => self
                .discover_single_installation(team_id, installed_by_user_id)
                .await?
                .ok_or(GithubAppError::NoInstallation),
        }
    }

    pub async fn link_review_identity(
        &self,
        team_id: Uuid,
        installation_id: i64,
        user_id: &str,
        login: &str,
    ) -> Result<GithubInstallation, GithubAppError> {
        self.repo
            .link_review_identity(&self.db, team_id, installation_id, user_id, login)
            .await
    }
}
