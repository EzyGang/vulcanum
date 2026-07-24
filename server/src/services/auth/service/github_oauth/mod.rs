mod client;
mod login;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::auth::errors::AuthError;
use crate::models::auth::model::{GithubCallbackResult, TeamPrincipal};
use crate::services::auth::service::AuthService;
use crate::services::github_app::service::{GithubAppManager, GithubInstallState};

const GITHUB_OAUTH_STATE_TTL_MINUTES: i64 = 10;
const DEFAULT_GITHUB_LOGIN_RETURN_TO: &str = "/login";
const DEFAULT_GITHUB_LINK_RETURN_TO: &str = "/settings?tab=github";

#[derive(Deserialize, Serialize)]
#[serde(tag = "purpose", rename_all = "snake_case")]
enum GithubOAuthState {
    Login {
        return_to: String,
    },
    LinkReviewIdentity {
        return_to: String,
        team_id: Uuid,
        installation_id: i64,
    },
}

impl AuthService {
    pub async fn github_authorize_url(&self, return_to: Option<&str>) -> Result<String, AuthError> {
        if self.is_single_user {
            return Err(AuthError::InvalidToken);
        }

        let return_to = validate_return_to(return_to)
            .unwrap_or(DEFAULT_GITHUB_LOGIN_RETURN_TO)
            .to_owned();
        self.github_oauth_url(GithubOAuthState::Login { return_to })
    }

    pub async fn github_link_authorize_url(
        &self,
        principal: &TeamPrincipal,
        return_to: Option<&str>,
    ) -> Result<String, AuthError> {
        if !self.is_single_user {
            return Err(AuthError::InvalidToken);
        }

        let team_id = self.teams.resolve_team(principal, true).await?;
        let installation = self
            .github_repo
            .get_installation(&self.db, team_id)
            .await?
            .ok_or(crate::models::github_app::errors::GithubAppError::NoInstallation)?;
        let return_to = validate_return_to(return_to)
            .unwrap_or(DEFAULT_GITHUB_LINK_RETURN_TO)
            .to_owned();
        self.github_oauth_url(GithubOAuthState::LinkReviewIdentity {
            return_to,
            team_id,
            installation_id: installation.id,
        })
    }

    pub(crate) async fn complete_github_installation_authorization(
        &self,
        github: &GithubAppManager,
        install_state: GithubInstallState,
        code: &str,
    ) -> Result<String, AuthError> {
        let token = self.exchange_github_code(code).await?;
        let github_user = self.fetch_github_user(&token).await?;
        let provider_user_id = github_user.id.to_string();

        if let Some(user_id) = install_state.user_id.as_deref() {
            self.teams
                .repo
                .upsert_identity(
                    &self.teams.db,
                    user_id,
                    "github",
                    &provider_user_id,
                    &github_user.login,
                )
                .await?;
        }

        let installation = github
            .connect_single_installation(install_state.team_id, install_state.user_id.as_deref())
            .await?;

        if self.is_single_user {
            github
                .link_review_identity(
                    install_state.team_id,
                    installation.id,
                    &provider_user_id,
                    &github_user.login,
                )
                .await?;
        }

        Ok(DEFAULT_GITHUB_LINK_RETURN_TO.to_owned())
    }

    pub async fn github_callback(
        &self,
        code: &str,
        state: &str,
    ) -> Result<GithubCallbackResult, AuthError> {
        let oauth_state = self.consume_github_oauth_state(state)?;
        if matches!(oauth_state, GithubOAuthState::Login { .. }) == self.is_single_user {
            return Err(AuthError::InvalidToken);
        }

        let token = self.exchange_github_code(code).await?;
        let github_user = self.fetch_github_user(&token).await?;
        match oauth_state {
            GithubOAuthState::Login { return_to } => {
                self.complete_github_login(github_user, return_to).await
            }
            GithubOAuthState::LinkReviewIdentity {
                return_to,
                team_id,
                installation_id,
            } => {
                self.github_repo
                    .link_review_identity(
                        &self.db,
                        team_id,
                        installation_id,
                        &github_user.id.to_string(),
                        &github_user.login,
                    )
                    .await?;
                Ok(GithubCallbackResult::IdentityLinked { return_to })
            }
        }
    }

    fn github_oauth_url(&self, oauth_state: GithubOAuthState) -> Result<String, AuthError> {
        let client_id = self
            .github_oauth_client_id
            .as_ref()
            .ok_or(AuthError::InvalidToken)?;
        let redirect_url = self
            .github_oauth_redirect_url
            .as_ref()
            .ok_or(AuthError::InvalidToken)?;
        let state = vulcanum_shared::crypto::generate_alphanumeric_string(32);
        let payload = serde_json::to_string(&oauth_state).map_err(|_| AuthError::InvalidToken)?;
        self.token_store
            .insert(&state, &payload, GITHUB_OAUTH_STATE_TTL_MINUTES);

        let mut url = url::Url::parse("https://github.com/login/oauth/authorize")
            .map_err(|_| AuthError::InvalidToken)?;
        url.query_pairs_mut()
            .append_pair("client_id", client_id)
            .append_pair("redirect_uri", redirect_url)
            .append_pair("scope", "read:user user:email")
            .append_pair("state", &state)
            .append_pair("allow_signup", "true")
            .append_pair("prompt", "select_account");

        Ok(url.to_string())
    }

    fn consume_github_oauth_state(&self, state: &str) -> Result<GithubOAuthState, AuthError> {
        self.token_store
            .consume(state)
            .and_then(|payload| serde_json::from_str::<GithubOAuthState>(&payload).ok())
            .ok_or(AuthError::InvalidToken)
    }
}

#[must_use]
pub(crate) fn validate_return_to(return_to: Option<&str>) -> Option<&str> {
    let return_to = return_to?;
    if !return_to.starts_with('/') || return_to.starts_with("//") {
        return None;
    }
    if return_to.contains('\\') || return_to.contains('#') {
        return None;
    }
    if return_to.chars().any(char::is_control) {
        return None;
    }

    let path = return_to.split('?').next().unwrap_or(return_to);
    if path.split('/').any(|segment| segment == "..") {
        return None;
    }

    Some(return_to)
}
