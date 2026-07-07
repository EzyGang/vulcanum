use std::collections::HashMap;
use std::path::Path;

use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::WorkspaceRepo;

pub async fn checkout_pull_request(
    workspace_dir: &Path,
    repos: &[WorkspaceRepo],
    repo_full_name: &str,
    pr_url: &str,
    command_env: &HashMap<String, String>,
) -> Result<(), HarnessError> {
    let repo = match repos.iter().find(|repo| repo.full_name == repo_full_name) {
        Some(repo) => repo,
        None => {
            return Err(HarnessError::Install(format!(
                "pull request repo {repo_full_name} was not cloned"
            )));
        }
    };
    let pull_request = match parse_github_pr_url(pr_url) {
        Some(pull_request) if pull_request.matches_repo(repo_full_name) => pull_request,
        Some(_) => {
            return Err(HarnessError::Install(format!(
                "pull request URL {pr_url} does not belong to repo {repo_full_name}"
            )));
        }
        None => {
            return Err(HarnessError::Install(format!(
                "invalid GitHub pull request URL: {pr_url}"
            )));
        }
    };
    let repo_dir = workspace_dir.join(&repo.relative_path);

    match run_checkout_command(&repo_dir, "gh", &["pr", "checkout", pr_url], command_env).await {
        Ok(()) => Ok(()),
        Err(e) => {
            tracing::warn!(
                repo = %repo_full_name,
                pr_url = %pr_url,
                error = %e,
                "gh pr checkout failed, falling back to git pull ref checkout"
            );
            checkout_pull_ref(&repo_dir, pull_request.number, command_env).await
        }
    }
}

#[must_use]
pub(crate) fn checkout_branch_name(pr_number: i64) -> String {
    format!("vulcanum-pr-{pr_number}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GithubPullRequest {
    owner: String,
    repo: String,
    number: i64,
}

impl GithubPullRequest {
    fn matches_repo(&self, full_name: &str) -> bool {
        let Some((owner, repo)) = full_name.split_once('/') else {
            return false;
        };

        self.owner.eq_ignore_ascii_case(owner) && self.repo.eq_ignore_ascii_case(repo)
    }
}

#[cfg(test)]
#[must_use]
pub(crate) fn parse_github_pr_number(pr_url: &str) -> Option<i64> {
    parse_github_pr_url(pr_url).map(|pull_request| pull_request.number)
}

#[must_use]
pub(crate) fn parse_github_pr_url(pr_url: &str) -> Option<GithubPullRequest> {
    let trimmed = pr_url.split(['?', '#']).next()?.trim_end_matches('/');
    let stripped = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))?;
    let (host, path) = stripped.split_once('/')?;

    if !host.eq_ignore_ascii_case("github.com") {
        return None;
    }

    let mut segments = path.split('/');
    let owner = segments.next()?.to_owned();
    let repo = segments.next()?.to_owned();
    let pull_segment = segments.next()?;
    let number = segments.next()?;

    if owner.is_empty()
        || repo.is_empty()
        || pull_segment != "pull"
        || number.is_empty()
        || segments.next().is_some()
    {
        return None;
    }

    let number = number.parse::<i64>().ok()?;
    (number > 0).then_some(GithubPullRequest {
        owner,
        repo,
        number,
    })
}

async fn checkout_pull_ref(
    repo_dir: &Path,
    pr_number: i64,
    command_env: &HashMap<String, String>,
) -> Result<(), HarnessError> {
    let branch = checkout_branch_name(pr_number);
    let pull_ref = format!("pull/{pr_number}/head:{branch}");

    run_checkout_command(
        repo_dir,
        "git",
        &["fetch", "origin", pull_ref.as_str()],
        command_env,
    )
    .await?;
    run_checkout_command(repo_dir, "git", &["checkout", branch.as_str()], command_env).await
}

async fn run_checkout_command(
    repo_dir: &Path,
    program: &str,
    args: &[&str],
    command_env: &HashMap<String, String>,
) -> Result<(), HarnessError> {
    let mut command = tokio::process::Command::new(program);
    command
        .args(args)
        .current_dir(repo_dir)
        .envs(command_env)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());

    let output = command
        .output()
        .await
        .map_err(|e| HarnessError::Install(format!("failed to run {program}: {e}")))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(HarnessError::Install(format!(
        "{program} {} failed: {stderr}",
        args.join(" ")
    )))
}
