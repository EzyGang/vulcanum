use serde::Deserialize;

use crate::models::auth::errors::AuthError;
use crate::services::auth::service::AuthService;

#[derive(Deserialize)]
pub(crate) struct GithubUserResponse {
    pub(crate) id: u64,
    pub(crate) login: String,
    pub(crate) email: Option<String>,
}

#[derive(Deserialize)]
struct GithubTokenResponse {
    access_token: String,
}

impl AuthService {
    pub(super) async fn exchange_github_code(&self, code: &str) -> Result<String, AuthError> {
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

        let response = self
            .github_oauth_http
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
            .map_err(|e| AuthError::GithubOAuth(format!("token exchange request: {e}")))?;
        let status = response.status();
        if !status.is_success() {
            return Err(AuthError::GithubOAuth(format!(
                "token exchange returned HTTP {status}"
            )));
        }

        let token: GithubTokenResponse = response
            .json()
            .await
            .map_err(|e| AuthError::GithubOAuth(format!("token exchange response: {e}")))?;
        Ok(token.access_token)
    }

    pub(super) async fn fetch_github_user(
        &self,
        token: &str,
    ) -> Result<GithubUserResponse, AuthError> {
        let response = self
            .github_oauth_http
            .get("https://api.github.com/user")
            .bearer_auth(token)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "vulcanum")
            .send()
            .await
            .map_err(|e| AuthError::GithubOAuth(format!("user request: {e}")))?;
        let status = response.status();
        if !status.is_success() {
            return Err(AuthError::GithubOAuth(format!(
                "user request returned HTTP {status}"
            )));
        }
        response
            .json()
            .await
            .map_err(|e| AuthError::GithubOAuth(format!("user response: {e}")))
    }
}
