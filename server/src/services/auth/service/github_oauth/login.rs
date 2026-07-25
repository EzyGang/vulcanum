use crate::models::auth::errors::AuthError;
use crate::models::auth::model::GithubCallbackResult;
use crate::services::auth::service::github_oauth::client::GithubUserResponse;
use crate::services::auth::service::AuthService;

impl AuthService {
    pub(super) async fn complete_github_login(
        &self,
        github_user: GithubUserResponse,
        return_to: String,
    ) -> Result<GithubCallbackResult, AuthError> {
        let provider_user_id = github_user.id.to_string();
        let user = match self
            .teams
            .repo
            .find_identity(&self.teams.db, "github", &provider_user_id)
            .await?
        {
            Some(identity) => self.users.find_user_by_id(&identity.user_id).await?,
            None => {
                let email = github_user.email.unwrap_or_else(|| {
                    format!(
                        "{}+{}@users.noreply.github.com",
                        github_user.login, github_user.id
                    )
                });
                self.users.find_or_create_user(&email).await?
            }
        };

        self.teams
            .repo
            .upsert_identity(
                &self.teams.db,
                &user.id,
                "github",
                &provider_user_id,
                &github_user.login,
            )
            .await?;
        self.teams
            .ensure_personal_team(&user.id, &github_user.login)
            .await?;
        self.users.update_last_login(&user.id).await?;

        let token_pair = self.issue_user_token_pair(&user.id).await?;
        Ok(GithubCallbackResult::Login {
            token_pair,
            return_to,
        })
    }
}
