use std::collections::{HashMap, HashSet};
use std::path::Path;

use tokio::fs;

use vulcanum_shared::api_types::{AgentBackend, JobRepo};
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::WorkspaceRepo;

use crate::isolation::workspace::context::surface_repo_context;

pub async fn clone_repo(
    url: &str,
    dest: &Path,
    command_env: &HashMap<String, String>,
) -> Result<(), HarnessError> {
    let mut command = tokio::process::Command::new("git");
    command
        .args(["clone", url])
        .arg(dest)
        .envs(command_env)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());

    let output = command
        .output()
        .await
        .map_err(|e| HarnessError::Install(format!("failed to run git clone: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let safe_stderr = stderr.replace(url, &redact_url_credentials(url));
        return Err(HarnessError::Install(format!(
            "git clone failed: {safe_stderr}"
        )));
    }

    Ok(())
}

pub async fn prepare_repos(
    workdir: &Path,
    repos: &[JobRepo],
    command_env: &HashMap<String, String>,
    agent_backend: AgentBackend,
) -> Result<Vec<WorkspaceRepo>, HarnessError> {
    let workspace_dir = workdir.join("workspace");
    fs::create_dir_all(&workspace_dir)
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to create workspace dir: {e}")))?;

    let workspace_repos = workspace_repos_from_job_repos(repos);
    for repo in &workspace_repos {
        clone_repo(
            &repo.url,
            &workspace_dir.join(&repo.relative_path),
            command_env,
        )
        .await?;
    }

    surface_repo_context(&workspace_dir, &workspace_repos, agent_backend).await?;
    Ok(workspace_repos)
}

#[must_use]
pub(crate) fn workspace_repos_from_job_repos(repos: &[JobRepo]) -> Vec<WorkspaceRepo> {
    let mut seen_dirs: HashSet<String> = HashSet::new();
    repos
        .iter()
        .map(|repo| WorkspaceRepo {
            full_name: repo.full_name.clone(),
            url: repo.url.clone(),
            relative_path: unique_repo_dir(&repo.full_name, &mut seen_dirs),
        })
        .collect()
}

#[must_use]
pub fn workspace_prompt_prefix(repos: &[WorkspaceRepo]) -> String {
    if repos.is_empty() {
        return String::new();
    }

    let repo_lines = repos
        .iter()
        .map(|repo| format!("- {}: ./{}", repo.full_name, repo.relative_path))
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        "Workspace layout:\n{repo_lines}\n\nThis wrapper workspace is not itself a repository. Run commands from the repository directory that matches the files you are changing. Before making changes, read the aggregated ./AGENTS.md plus any AGENTS.md files from the relevant repository root down to the changed directories. Before finishing, run the formatter, validation, and test commands required by those instructions for every changed repository.\n\n"
    )
}

#[must_use]
pub(crate) fn repo_dir_name(full_name: &str) -> String {
    let repo_name = full_name.rsplit('/').next().unwrap_or(full_name);
    sanitize_repo_dir(repo_name)
}

#[must_use]
pub(crate) fn redact_url_credentials(url: &str) -> String {
    let Some(rest) = url.strip_prefix("https://") else {
        return url.to_owned();
    };

    let (authority, path) = match rest.split_once('/') {
        Some((authority, path)) => (authority, Some(path)),
        None => (rest, None),
    };
    let Some((_, host)) = authority.rsplit_once('@') else {
        return url.to_owned();
    };

    match path {
        Some(path) => format!("https://<redacted>@{host}/{path}"),
        None => format!("https://<redacted>@{host}"),
    }
}

fn unique_repo_dir(full_name: &str, seen: &mut HashSet<String>) -> String {
    let base = repo_dir_name(full_name);
    let mut candidate = base.clone();
    let mut suffix = 2;
    while seen.contains(&candidate) {
        candidate = format!("{base}-{suffix}");
        suffix += 1;
    }
    seen.insert(candidate.clone());
    candidate
}

fn sanitize_repo_dir(repo_name: &str) -> String {
    let base = repo_name
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '-',
        })
        .collect::<String>()
        .trim_matches('-')
        .to_lowercase();

    match base.is_empty() {
        true => "repo".to_owned(),
        false => base,
    }
}
