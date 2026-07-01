use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use tokio::fs;

use vulcanum_shared::api_types::{
    AgentBackend, AgentConfigPayload, JobRepo, OpenCodeProviderConfig, WorkRunType,
};
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::WorkspaceRepo;

use crate::daemon::job::tool::finish_tool::{finish_run_tool_ts, omp_finish_run_tool_ts};

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

#[must_use]
pub(crate) fn omp_environment_vars(home: &str, tmpdir: &str) -> HashMap<String, String> {
    let config_home = env_path(home, ".omp");
    let state_home = env_path(home, ".local/state/omp");
    HashMap::from([
        ("PI_CONFIG_HOME".to_owned(), config_home.clone()),
        (
            "PI_DATA_HOME".to_owned(),
            env_path(home, ".local/share/omp"),
        ),
        ("PI_STATE_HOME".to_owned(), state_home.clone()),
        (
            "PI_SESSION_DIR".to_owned(),
            env_path(&config_home, "sessions"),
        ),
        ("PI_LOG_DIR".to_owned(), env_path(&state_home, "logs")),
        ("PI_TMPDIR".to_owned(), tmpdir.to_owned()),
        (
            "PI_PERMISSION_DEFAULT".to_owned(),
            "allow_always".to_owned(),
        ),
    ])
}

#[must_use]
pub(crate) fn container_path(workdir: &Path, container_workdir: &str, path: &Path) -> String {
    let path_text = path.to_string_lossy();
    if path_text.starts_with(container_workdir) {
        return path_text.to_string();
    }

    match path.strip_prefix(workdir) {
        Ok(relative_path) => env_path(
            container_workdir,
            relative_path.to_string_lossy().replace('\\', "/").as_str(),
        ),
        Err(_) => path_text.to_string(),
    }
}

#[cfg(windows)]
fn env_path(base: &str, suffix: &str) -> String {
    let separator = match base.contains('\\') && !base.contains('/') {
        true => "\\",
        false => "/",
    };
    format!(
        "{}{}{}",
        base.trim_end_matches(['/', '\\']),
        separator,
        suffix.replace('/', separator)
    )
}

#[cfg(not(windows))]
fn env_path(base: &str, suffix: &str) -> String {
    format!("{}/{}", base.trim_end_matches('/'), suffix)
}

pub async fn write_agent_files(
    workdir: &Path,
    agents_md: &str,
    agent_backend: AgentBackend,
    agent_config: &AgentConfigPayload,
    work_type: WorkRunType,
) -> Result<(), HarnessError> {
    match (agent_backend, agent_config) {
        (
            AgentBackend::OpenCode,
            AgentConfigPayload::OpenCode {
                providers,
                model,
                small_model,
                auth_content: _,
            },
        ) => {
            write_opencode_env_files(
                workdir,
                agents_md,
                providers,
                model.as_deref(),
                small_model.as_deref(),
            )
            .await?;
            write_opencode_finish_run_tool(workdir, work_type).await?;
        }
        (AgentBackend::OmpRpc, AgentConfigPayload::OmpRpc { config_yml }) => {
            write_omp_env_files(workdir, config_yml.as_deref()).await?;
            write_omp_finish_run_tool(workdir, work_type).await?;
        }
        (backend, payload) => {
            return Err(HarnessError::Crash(format!(
                "agent config payload {:?} does not match backend {}",
                payload.backend(),
                backend.as_str()
            )));
        }
    }

    Ok(())
}

async fn write_opencode_env_files(
    workdir: &Path,
    agents_md: &str,
    providers: &HashMap<String, OpenCodeProviderConfig>,
    model: Option<&str>,
    small_model: Option<&str>,
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

    let mut config = serde_json::Map::new();
    config.insert(
        "permission".to_owned(),
        serde_json::json!({
            "*": "allow",
            "question": "deny",
        }),
    );
    if !providers.is_empty() {
        config.insert("provider".to_owned(), serde_json::json!(providers));
    }
    if let Some(model) = model {
        config.insert("model".to_owned(), serde_json::json!(model));
    }
    if let Some(small_model) = small_model {
        config.insert("small_model".to_owned(), serde_json::json!(small_model));
    }

    fs::write(
        config_dir.join("opencode.json"),
        serde_json::Value::Object(config).to_string(),
    )
    .await
    .map_err(|e| HarnessError::Crash(format!("failed to write opencode.json: {e}")))?;

    Ok(())
}

async fn write_omp_env_files(workdir: &Path, config_yml: Option<&str>) -> Result<(), HarnessError> {
    let config_dir = workdir.join("home").join(".omp").join("agent");
    fs::create_dir_all(&config_dir)
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to create OMP config dir: {e}")))?;

    if let Some(config_yml) = config_yml {
        if !config_yml.is_empty() {
            fs::write(config_dir.join("config.yml"), config_yml)
                .await
                .map_err(|e| HarnessError::Crash(format!("failed to write OMP config.yml: {e}")))?;
        }
    }

    Ok(())
}

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

async fn write_opencode_finish_run_tool(
    workdir: &Path,
    work_type: WorkRunType,
) -> Result<(), HarnessError> {
    let tools_dir = workdir
        .join("home")
        .join(".config")
        .join("opencode")
        .join("tools");
    fs::create_dir_all(&tools_dir)
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to create tools dir: {e}")))?;

    fs::write(
        tools_dir.join("finish_run.ts"),
        finish_run_tool_ts(work_type),
    )
    .await
    .map_err(|e| HarnessError::Crash(format!("failed to write finish_run tool: {e}")))?;

    Ok(())
}

async fn write_omp_finish_run_tool(
    workdir: &Path,
    work_type: WorkRunType,
) -> Result<(), HarnessError> {
    let tools_dir = workdir
        .join("workspace")
        .join(".omp")
        .join("tools")
        .join("finish_run");
    fs::create_dir_all(&tools_dir)
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to create OMP tools dir: {e}")))?;

    fs::write(
        tools_dir.join("index.ts"),
        omp_finish_run_tool_ts(work_type),
    )
    .await
    .map_err(|e| HarnessError::Crash(format!("failed to write OMP finish_run tool: {e}")))?;

    Ok(())
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

pub(crate) async fn surface_repo_context(
    workspace_dir: &Path,
    repos: &[WorkspaceRepo],
    agent_backend: AgentBackend,
) -> Result<(), HarnessError> {
    let mut aggregate = String::new();
    let skills_dir = match agent_backend {
        AgentBackend::OpenCode => workspace_dir.join(".agents").join("skills"),
        AgentBackend::OmpRpc => workspace_dir.join(".omp").join("skills"),
    };
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

        for skills_root in skill_roots(&repo_dir) {
            copy_skills_first_wins(&skills_root, &skills_dir, &mut copied_skills, repo).await?;
        }
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

fn skill_roots(repo_dir: &Path) -> [PathBuf; 4] {
    [
        repo_dir.join(".agents").join("skills"),
        repo_dir.join(".claude").join("skills"),
        repo_dir.join(".codex").join("skills"),
        repo_dir.join(".omp").join("skills"),
    ]
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
    let mut skill_entries = Vec::new();

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
        skill_entries.push((
            entry.file_name().to_string_lossy().to_string(),
            entry.path(),
        ));
    }

    skill_entries.sort_by(|(left_name, _), (right_name, _)| left_name.cmp(right_name));

    for (skill_name, source_path) in skill_entries {
        if copied.contains(&skill_name) {
            tracing::warn!(
                repo = %repo.full_name,
                skill = %skill_name,
                "skipping duplicate repo skill"
            );
            continue;
        }
        copy_dir(&source_path, &target.join(&skill_name)).await?;
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
