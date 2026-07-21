use octocrab::models::InstallationId;
use uuid::Uuid;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::{GithubAppManager, InstallationToken, RepoInfo};
use crate::util::github::parse_github_repo;

impl GithubAppManager {
    pub async fn list_repos(&self, team_id: Uuid) -> Result<Vec<RepoInfo>, GithubAppError> {
        let installation = self
            .repo
            .get_installation(&self.db, team_id)
            .await?
            .ok_or(GithubAppError::NoInstallation)?;

        let octo = self.app_octocrab()?;
        let installation_client = octo
            .installation(InstallationId(installation.github_installation_id as u64))
            .map_err(|e| GithubAppError::Api(format!("installation client: {e}")))?;

        let repos = installation_client
            .get::<octocrab::Page<octocrab::models::Repository>, _, ()>(
                "/installation/repositories",
                None::<&()>,
            )
            .await
            .map_err(|e| GithubAppError::Api(format!("list_repos: {e}")))?;

        let all_repos = installation_client
            .all_pages(repos)
            .await
            .map_err(|e| GithubAppError::Api(format!("list_repos pagination: {e}")))?;

        let infos = all_repos
            .into_iter()
            .map(|r| RepoInfo {
                owner: r.owner.map(|o| o.login).unwrap_or_default(),
                name: r.name,
                full_name: r.full_name.unwrap_or_default(),
            })
            .collect();

        Ok(infos)
    }

    pub async fn generate_installation_token_for_repos(
        &self,
        team_id: Uuid,
        repo_full_names: &[String],
    ) -> Result<InstallationToken, GithubAppError> {
        let installation = self
            .repo
            .get_installation(&self.db, team_id)
            .await?
            .ok_or(GithubAppError::NoInstallation)?;
        self.mint_installation_token(installation.github_installation_id, repo_full_names)
            .await
    }

    pub(crate) async fn generate_installation_token_for_installation(
        &self,
        team_id: Uuid,
        github_installation_id: i64,
        repo_full_names: &[String],
    ) -> Result<InstallationToken, GithubAppError> {
        let current_team = self
            .repo
            .find_team_id_by_github_installation(&self.db, github_installation_id)
            .await?;
        if current_team != Some(team_id) {
            return Err(GithubAppError::NoInstallation);
        }
        self.mint_installation_token(github_installation_id, repo_full_names)
            .await
    }

    async fn mint_installation_token(
        &self,
        github_installation_id: i64,
        repo_full_names: &[String],
    ) -> Result<InstallationToken, GithubAppError> {
        let mut repo_names = Vec::with_capacity(repo_full_names.len());
        for full_name in repo_full_names {
            let repo = parse_github_repo(full_name)
                .ok_or_else(|| GithubAppError::InvalidRepoIdentifier(full_name.clone()))?;
            repo_names.push(repo.name().to_owned());
        }

        let octo = self.app_octocrab()?;
        let route = format!("/app/installations/{github_installation_id}/access_tokens");
        let body = serde_json::json!({
            "repositories": repo_names,
            "permissions": {
                "contents": "write",
                "pull_requests": "write"
            }
        });
        let response: octocrab::models::InstallationToken = octo
            .post(&route, Some(&body))
            .await
            .map_err(|e| GithubAppError::Api(format!("token mint failed: {e}")))?;
        let expires_at = response
            .expires_at
            .as_ref()
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value: chrono::DateTime<chrono::FixedOffset>| value.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        Ok(InstallationToken {
            token: response.token,
            expires_at,
        })
    }
}
