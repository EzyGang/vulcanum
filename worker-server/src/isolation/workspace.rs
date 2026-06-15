use std::collections::HashSet;
use std::path::{Path, PathBuf};

use tokio::fs;

use vulcanum_shared::api_types::JobRepo;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::WorkspaceRepo;

use crate::daemon::job::finish_tool::FINISH_RUN_TOOL_TS;

#[must_use]
pub fn container_name(workdir: &Path) -> String {
    format!(
        "vulcanum-{}",
        workdir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("job")
    )
}

pub async fn write_env_files(
    workdir: &Path,
    agents_md: &str,
    generated_opencode_config: &str,
) -> Result<(), HarnessError> {
    let config_dir = workdir.join("home").join(".config").join("opencode");
    fs::create_dir_all(&config_dir)
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to create config dir: {e}")))?;

    if !agents_md.is_empty() {
        fs::write(config_dir.join("AGENTS.md"), agents_md)
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to write AGENTS.md: {e}")))?;
    }

    if !generated_opencode_config.is_empty() {
        fs::write(config_dir.join("opencode.json"), generated_opencode_config)
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to write opencode.json: {e}")))?;
    }

    Ok(())
}

#[must_use]
pub fn authenticated_repo_url(repo_url: &str, token: Option<&str>) -> String {
    if let Some(t) = token {
        if let Some(host_path) = repo_url.strip_prefix("https://") {
            return format!("https://x-access-token:{t}@{host_path}");
        }
    }
    repo_url.to_owned()
}

pub async fn clone_repo(url: &str, dest: &Path) -> Result<(), HarnessError> {
    let output = tokio::process::Command::new("git")
        .args([
            "-c",
            "credential.helper=",
            "-c",
            "core.askPass=",
            "clone",
            url,
        ])
        .arg(dest)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
        .map_err(|e| HarnessError::Install(format!("failed to run git clone: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HarnessError::Install(format!("git clone failed: {stderr}")));
    }

    Ok(())
}

pub async fn prepare_repos(
    workdir: &Path,
    repos: &[JobRepo],
    token: Option<&str>,
) -> Result<Vec<WorkspaceRepo>, HarnessError> {
    let workspace_dir = workdir.join("workspace");
    fs::create_dir_all(&workspace_dir)
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to create workspace dir: {e}")))?;

    let mut seen_dirs: HashSet<String> = HashSet::new();
    let mut workspace_repos: Vec<WorkspaceRepo> = Vec::new();
    for repo in repos {
        let relative_path = unique_repo_dir(&repo.full_name, &mut seen_dirs);
        let clone_url = authenticated_repo_url(&repo.url, token);
        clone_repo(&clone_url, &workspace_dir.join(&relative_path)).await?;
        workspace_repos.push(WorkspaceRepo {
            full_name: repo.full_name.clone(),
            url: repo.url.clone(),
            relative_path,
        });
    }

    surface_repo_context(&workspace_dir, &workspace_repos).await?;
    Ok(workspace_repos)
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
        "Workspace layout:\n{repo_lines}\n\nRun commands from the repository directory that matches the file you are changing. Read the aggregated AGENTS.md in this workspace before making changes.\n\n"
    )
}

pub async fn write_finish_run_tool(workdir: &Path) -> Result<(), HarnessError> {
    let tools_dir = workdir.join("home").join(".opencode").join("tools");
    fs::create_dir_all(&tools_dir)
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to create tools dir: {e}")))?;

    fs::write(tools_dir.join("finish_run.ts"), FINISH_RUN_TOOL_TS)
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to write finish_run tool: {e}")))?;

    Ok(())
}

fn unique_repo_dir(full_name: &str, seen: &mut HashSet<String>) -> String {
    let base = full_name
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '-',
        })
        .collect::<String>()
        .trim_matches('-')
        .to_lowercase();
    let base = match base.is_empty() {
        true => "repo".to_owned(),
        false => base,
    };
    let mut candidate = base.clone();
    let mut suffix = 2;
    while seen.contains(&candidate) {
        candidate = format!("{base}-{suffix}");
        suffix += 1;
    }
    seen.insert(candidate.clone());
    candidate
}

async fn surface_repo_context(
    workspace_dir: &Path,
    repos: &[WorkspaceRepo],
) -> Result<(), HarnessError> {
    let mut aggregate = String::new();
    let skills_dir = workspace_dir.join(".agents").join("skills");
    fs::create_dir_all(&skills_dir)
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to create skills dir: {e}")))?;
    let mut copied_skills: HashSet<String> = HashSet::new();

    for repo in repos {
        let repo_dir = workspace_dir.join(&repo.relative_path);
        for file_name in ["AGENTS.md", "agents.md"] {
            let path = repo_dir.join(file_name);
            let Ok(contents) = fs::read_to_string(&path).await else {
                continue;
            };
            aggregate.push_str(&format!(
                "\n# Repository: {}\nPath: ./{}\n\n{}\n",
                repo.full_name, repo.relative_path, contents
            ));
            break;
        }

        let repo_skills_dir = repo_dir.join(".agents").join("skills");
        copy_skills_first_wins(&repo_skills_dir, &skills_dir, &mut copied_skills, repo).await?;
    }

    if !aggregate.is_empty() {
        fs::write(workspace_dir.join("AGENTS.md"), aggregate)
            .await
            .map_err(|e| {
                HarnessError::Crash(format!("failed to write workspace AGENTS.md: {e}"))
            })?;
    }

    Ok(())
}

async fn copy_skills_first_wins(
    source: &Path,
    target: &Path,
    copied: &mut HashSet<String>,
    repo: &WorkspaceRepo,
) -> Result<(), HarnessError> {
    let mut entries = match fs::read_dir(source).await {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to read skills dir: {e}")))?
    {
        let file_type = entry
            .file_type()
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to read skill file type: {e}")))?;
        if !file_type.is_dir() {
            continue;
        }
        let skill_name = entry.file_name().to_string_lossy().to_string();
        if copied.contains(&skill_name) {
            tracing::warn!(
                repo = %repo.full_name,
                skill = %skill_name,
                "skipping duplicate repo skill"
            );
            continue;
        }
        copy_dir(&entry.path(), &target.join(&skill_name)).await?;
        copied.insert(skill_name);
    }

    Ok(())
}

async fn copy_dir(source: &Path, target: &Path) -> Result<(), HarnessError> {
    fs::create_dir_all(target)
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to create copy target: {e}")))?;
    let mut stack: Vec<(PathBuf, PathBuf)> = vec![(source.to_path_buf(), target.to_path_buf())];

    while let Some((current_source, current_target)) = stack.pop() {
        let mut entries = fs::read_dir(&current_source)
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to read copy source: {e}")))?;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to read copy entry: {e}")))?
        {
            let target_path = current_target.join(entry.file_name());
            let file_type = entry
                .file_type()
                .await
                .map_err(|e| HarnessError::Crash(format!("failed to read copy file type: {e}")))?;
            if file_type.is_dir() {
                fs::create_dir_all(&target_path)
                    .await
                    .map_err(|e| HarnessError::Crash(format!("failed to create copy dir: {e}")))?;
                stack.push((entry.path(), target_path));
            } else if file_type.is_file() {
                fs::copy(entry.path(), &target_path)
                    .await
                    .map_err(|e| HarnessError::Crash(format!("failed to copy file: {e}")))?;
            }
        }
    }

    Ok(())
}
