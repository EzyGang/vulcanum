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
pub(crate) trait PullRequestCommentWriter: Send + Sync {
    async fn ensure_pull_request_comment(
        &self,
        team_id: Uuid,
        installation_id: i64,
        repo_full_name: &str,
        pr_number: i64,
        marker: &str,
        body: &str,
    ) -> Result<(), GithubAppError>;
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
impl PullRequestCommentWriter for GithubAppManager {
    async fn ensure_pull_request_comment(
        &self,
        team_id: Uuid,
        installation_id: i64,
        repo_full_name: &str,
        pr_number: i64,
        marker: &str,
        body: &str,
    ) -> Result<(), GithubAppError> {
        let current_team = self
            .repo
            .find_team_id_by_github_installation(&self.db, installation_id)
            .await?;
        if current_team != Some(team_id) {
            return Err(GithubAppError::NoInstallation);
        }
        let repo = parse_github_repo(repo_full_name)
            .ok_or_else(|| GithubAppError::InvalidRepoIdentifier(repo_full_name.to_owned()))?;
        let number = u64::try_from(pr_number)
            .map_err(|e| GithubAppError::Api(format!("invalid pull request number: {e}")))?;
        let client = self
            .app_octocrab()?
            .installation(InstallationId(installation_id as u64))
            .map_err(|e| GithubAppError::Api(format!("installation client: {e}")))?;
        let mut page = client
            .issues(repo.owner(), repo.name())
            .list_comments(number)
            .per_page(100)
            .send()
            .await
            .map_err(|e| GithubAppError::Api(format!("list pull request comments: {e}")))?;

        loop {
            if page.items.iter().any(|comment| {
                comment
                    .body
                    .as_deref()
                    .is_some_and(|body| body.contains(marker))
            }) {
                return Ok(());
            }
            page = match client
                .get_page(&page.next)
                .await
                .map_err(|e| GithubAppError::Api(format!("list pull request comments: {e}")))?
            {
                Some(page) => page,
                None => break,
            };
        }

        client
            .issues(repo.owner(), repo.name())
            .create_comment(number, format!("{body}\n\n{marker}"))
            .await
            .map(|_| ())
            .map_err(|e| GithubAppError::Api(format!("create pull request comment: {e}")))
    }
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
