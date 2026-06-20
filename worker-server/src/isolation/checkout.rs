use std::path::Path;

use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::WorkspaceRepo;

pub async fn checkout_pull_request(
    workspace_dir: &Path,
    repos: &[WorkspaceRepo],
    repo_full_name: &str,
    pr_url: &str,
    token: Option<&str>,
) -> Result<(), HarnessError> {
    let repo = match repos.iter().find(|repo| repo.full_name == repo_full_name) {
        Some(repo) => repo,
        None => {
            return Err(HarnessError::Install(format!(
                "pull request repo {repo_full_name} was not cloned"
            )));
        }
    };
    let pr_number = match parse_github_pr_number(pr_url) {
        Some(number) => number,
        None => {
            return Err(HarnessError::Install(format!(
                "invalid GitHub pull request URL: {pr_url}"
            )));
        }
    };
    let repo_dir = workspace_dir.join(&repo.relative_path);

    match run_checkout_command(&repo_dir, "gh", &["pr", "checkout", pr_url], token).await {
        Ok(()) => Ok(()),
        Err(e) => {
            tracing::warn!(
                repo = %repo_full_name,
                pr_url = %pr_url,
                error = %e,
                "gh pr checkout failed, falling back to git pull ref checkout"
            );
            checkout_pull_ref(&repo_dir, pr_number, token).await
        }
    }
}

#[must_use]
pub(crate) fn checkout_branch_name(pr_number: i64) -> String {
    format!("vulcanum-pr-{pr_number}")
}

#[must_use]
pub(crate) fn parse_github_pr_number(pr_url: &str) -> Option<i64> {
    let trimmed = pr_url.split(['?', '#']).next()?.trim_end_matches('/');
    let (_, number) = trimmed.rsplit_once("/pull/")?;

    if number.contains('/') {
        return None;
    }

    number.parse::<i64>().ok()
}

async fn checkout_pull_ref(
    repo_dir: &Path,
    pr_number: i64,
    token: Option<&str>,
) -> Result<(), HarnessError> {
    let branch = checkout_branch_name(pr_number);
    let pull_ref = format!("pull/{pr_number}/head:{branch}");

    run_checkout_command(
        repo_dir,
        "git",
        &["fetch", "origin", pull_ref.as_str()],
        token,
    )
    .await?;
    run_checkout_command(repo_dir, "git", &["checkout", branch.as_str()], token).await
}

async fn run_checkout_command(
    repo_dir: &Path,
    program: &str,
    args: &[&str],
    token: Option<&str>,
) -> Result<(), HarnessError> {
    let mut command = tokio::process::Command::new(program);
    command
        .args(args)
        .current_dir(repo_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());

    if let Some(token) = token {
        command.env("GITHUB_TOKEN", token);
        command.env("GH_TOKEN", token);
    }

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
