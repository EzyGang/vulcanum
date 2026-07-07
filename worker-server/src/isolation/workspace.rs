mod context;
mod repos;

use std::collections::HashMap;
use std::path::Path;

use tokio::fs;

use vulcanum_shared::api_types::{
    AgentBackend, AgentConfigPayload, OpenCodeProviderConfig, WorkRunType,
};
use vulcanum_shared::runtime::errors::HarnessError;

use crate::daemon::job::tool::finish_tool::{finish_run_tool_ts, omp_finish_run_tool_ts};
#[cfg(test)]
pub(crate) use crate::isolation::workspace::context::surface_repo_context;
#[cfg(test)]
pub(crate) use crate::isolation::workspace::repos::{
    clone_repo, redact_url_credentials, repo_dir_name, workspace_repos_from_job_repos,
};
pub(crate) use crate::isolation::workspace::repos::{prepare_repos, workspace_prompt_prefix};

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
