use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tokio::fs;

use vulcanum_shared::api::wire::AgentBackend;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::WorkspaceRepo;

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
    let mut copied_skills: HashMap<String, CopiedSkillOrigin> = HashMap::new();

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

struct CopiedSkillOrigin {
    repo_full_name: String,
    source_path: PathBuf,
}

async fn copy_skills_first_wins(
    source: &Path,
    target: &Path,
    copied: &mut HashMap<String, CopiedSkillOrigin>,
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
        if let Some(origin) = copied.get(&skill_name) {
            log_duplicate_skill(repo, &skill_name, &source_path, origin);
            continue;
        }
        copy_dir(&source_path, &target.join(&skill_name)).await?;
        copied.insert(
            skill_name,
            CopiedSkillOrigin {
                repo_full_name: repo.full_name.clone(),
                source_path,
            },
        );
    }

    Ok(())
}

fn log_duplicate_skill(
    repo: &WorkspaceRepo,
    skill_name: &str,
    source_path: &Path,
    origin: &CopiedSkillOrigin,
) {
    match origin.repo_full_name == repo.full_name {
        true => tracing::debug!(
            repo = %repo.full_name,
            skill = %skill_name,
            source = %source_path.display(),
            existing_source = %origin.source_path.display(),
            "skipping duplicate repo skill"
        ),
        false => tracing::warn!(
            repo = %repo.full_name,
            existing_repo = %origin.repo_full_name,
            skill = %skill_name,
            source = %source_path.display(),
            existing_source = %origin.source_path.display(),
            "skipping duplicate repo skill from another repository"
        ),
    }
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
