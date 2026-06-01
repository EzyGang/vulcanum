use std::path::Path;

use tokio::fs;
use vulcanum_shared::runtime::errors::HarnessError;

pub async fn write_env_files(
    workdir: &Path,
    agents_md: &str,
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

    if !opencode_config.is_empty() {
        fs::write(config_dir.join("opencode.json"), opencode_config)
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to write opencode.json: {e}")))?;
    }

    Ok(())
}

pub async fn clone_repo(url: &str, dest: &Path) -> Result<(), HarnessError> {
    let output = tokio::process::Command::new("git")
        .arg("clone")
        .arg(url)
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
