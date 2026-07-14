use async_trait::async_trait;
use octocrab::models::{InstallationId, IssueState};
use uuid::Uuid;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::GithubAppManager;
use crate::util::github::parse_github_repo;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum PullRequestState {
    Open,
    Closed,
    Merged,
}

impl PullRequestState {
    #[must_use]
    pub(crate) const fn is_terminal(self) -> bool {
        matches!(self, Self::Closed | Self::Merged)
    }
}

#[async_trait]
pub(crate) trait PullRequestStateReader: Send + Sync {
    async fn pull_request_state(
        &self,
        team_id: Uuid,
        repo_full_name: &str,
        number: i64,
    ) -> Result<PullRequestState, GithubAppError>;
}

#[async_trait]
impl PullRequestStateReader for GithubAppManager {
    async fn pull_request_state(
        &self,
        team_id: Uuid,
        repo_full_name: &str,
        number: i64,
    ) -> Result<PullRequestState, GithubAppError> {
        let repo = parse_github_repo(repo_full_name)
            .ok_or_else(|| GithubAppError::InvalidRepoIdentifier(repo_full_name.to_owned()))?;
        let number = u64::try_from(number)
            .map_err(|e| GithubAppError::Api(format!("invalid pull request number: {e}")))?;
        let installation = self
            .repo
            .get_installation(&self.db, team_id)
            .await?
            .ok_or(GithubAppError::NoInstallation)?;
        let client = self
            .app_octocrab()?
            .installation(InstallationId(installation.github_installation_id as u64))
            .map_err(|e| GithubAppError::Api(format!("installation client: {e}")))?;
        let pull_request = client
            .pulls(repo.owner(), repo.name())
            .get(number)
            .await
            .map_err(|e| GithubAppError::Api(format!("get pull request: {e}")))?;

        if pull_request.merged == Some(true) || pull_request.merged_at.is_some() {
            return Ok(PullRequestState::Merged);
        }

        match pull_request.state {
            Some(IssueState::Open) => Ok(PullRequestState::Open),
            Some(IssueState::Closed) => Ok(PullRequestState::Closed),
            Some(_) | None => Err(GithubAppError::Api(
                "pull request response omitted a supported state".to_owned(),
            )),
        }
    }
}
