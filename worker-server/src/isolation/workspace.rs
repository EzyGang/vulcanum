use std::path::Path;

use tokio::fs;

use vulcanum_shared::runtime::errors::HarnessError;

use crate::daemon::job::finish_tool::FINISH_RUN_TOOL_TS;

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
    opencode_config: &str,
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

    if !opencode_config.is_empty() {
        fs::write(config_dir.join("opencode.user.json"), opencode_config)
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to write opencode.user.json: {e}")))?;
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
