use serde::{Deserialize, Serialize};

use crate::services::auth::errors::AuthError;
use crate::services::auth::model::GithubCallbackResult;
use crate::services::auth::service::AuthService;
const GITHUB_OAUTH_STATE_TTL_MINUTES: i64 = 10;
const DEFAULT_GITHUB_RETURN_TO: &str = "/login";

#[derive(Deserialize, Serialize)]
struct GithubOAuthState {
    return_to: String,
}

#[derive(Deserialize)]
struct GithubTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GithubUserResponse {
    id: u64,
    login: String,
    email: Option<String>,
}

impl AuthService {
    pub async fn github_authorize_url(&self, return_to: Option<&str>) -> Result<String, AuthError> {
        if self.is_single_user {
            return Err(AuthError::InvalidToken);
        }

        let client_id = self
            .github_oauth_client_id
            .as_ref()
            .ok_or(AuthError::InvalidToken)?;
        let redirect_url = self
            .github_oauth_redirect_url
            .as_ref()
            .ok_or(AuthError::InvalidToken)?;
        let state = vulcanum_shared::crypto::generate_alphanumeric_string(32);
        let return_to = validate_return_to(return_to).unwrap_or(DEFAULT_GITHUB_RETURN_TO);
        let oauth_state = GithubOAuthState {
            return_to: return_to.to_owned(),
        };
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

    pub async fn github_callback(
        &self,
        code: &str,
        state: &str,
    ) -> Result<GithubCallbackResult, AuthError> {
        if self.is_single_user {
            return Err(AuthError::InvalidToken);
        }

        let oauth_state = self
            .token_store
            .consume(state)
            .and_then(|payload| serde_json::from_str::<GithubOAuthState>(&payload).ok())
            .ok_or(AuthError::InvalidToken)?;

        let token = self.exchange_github_code(code).await?;
        let github_user = self.fetch_github_user(&token).await?;
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

        Ok(GithubCallbackResult {
            token_pair,
            return_to: oauth_state.return_to,
        })
    }

    async fn exchange_github_code(&self, code: &str) -> Result<String, AuthError> {
        let client_id = self
            .github_oauth_client_id
            .as_ref()
            .ok_or(AuthError::InvalidToken)?;
        let client_secret = self
            .github_oauth_client_secret
            .as_ref()
            .ok_or(AuthError::InvalidToken)?;
        let redirect_url = self
            .github_oauth_redirect_url
            .as_ref()
            .ok_or(AuthError::InvalidToken)?;

        let response = reqwest::Client::new()
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("code", code),
                ("redirect_uri", redirect_url.as_str()),
            ])
            .send()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let token: GithubTokenResponse =
            response.json().await.map_err(|_| AuthError::InvalidToken)?;
        Ok(token.access_token)
    }

    async fn fetch_github_user(&self, token: &str) -> Result<GithubUserResponse, AuthError> {
        reqwest::Client::new()
            .get("https://api.github.com/user")
            .bearer_auth(token)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "vulcanum")
            .send()
            .await
            .map_err(|_| AuthError::InvalidToken)?
            .json()
            .await
            .map_err(|_| AuthError::InvalidToken)
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
