use std::time::Duration;

use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;
use vulcanum_shared::api::wire::GitCommitAuthor;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::GithubAppManager;

const GITHUB_API_URL: &str = "https://api.github.com";
const GITHUB_API_TIMEOUT: Duration = Duration::from_secs(10);

impl GithubAppManager {
    pub(crate) async fn commit_author(&self) -> Option<GitCommitAuthor> {
        match self
            .commit_author_cache
            .get_or_try_init(|| self.fetch_commit_author())
            .await
        {
            Ok(author) => Some(author.clone()),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    app_slug = self.app_slug.as_deref().unwrap_or("<unset>"),
                    "GitHub App commit attribution is unavailable",
                );
                None
            }
        }
    }

    async fn fetch_commit_author(&self) -> Result<GitCommitAuthor, GithubAppError> {
        let app_slug = self
            .app_slug
            .as_deref()
            .filter(|slug| !slug.trim().is_empty())
            .ok_or(GithubAppError::NotConfigured)?;
        let login = format!("{app_slug}[bot]");
        let encoded_login =
            url::form_urlencoded::byte_serialize(login.as_bytes()).collect::<String>();
        let client = reqwest::Client::builder()
            .timeout(GITHUB_API_TIMEOUT)
            .build()
            .map_err(|e| GithubAppError::Api(format!("build bot identity client: {e}")))?;
        let user = client
            .get(format!("{GITHUB_API_URL}/users/{encoded_login}"))
            .header(ACCEPT, "application/vnd.github+json")
            .header(USER_AGENT, "vulcanum")
            .send()
            .await
            .map_err(|e| GithubAppError::Api(format!("get app bot identity: {e}")))?
            .error_for_status()
            .map_err(|e| GithubAppError::Api(format!("get app bot identity: {e}")))?
            .json::<GithubBotUser>()
            .await
            .map_err(|e| GithubAppError::Api(format!("decode app bot identity: {e}")))?;

        Ok(commit_author_for(user.id, &user.login))
    }
}

#[derive(Deserialize)]
struct GithubBotUser {
    id: u64,
    login: String,
}

pub(super) fn commit_author_for(id: u64, login: &str) -> GitCommitAuthor {
    GitCommitAuthor {
        name: login.to_owned(),
        email: format!("{id}+{login}@users.noreply.github.com"),
    }
}
