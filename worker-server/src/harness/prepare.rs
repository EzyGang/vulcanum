use std::path::Path;

use tokio::fs;
use vulcanum_shared::runtime::errors::HarnessError;

const FINISH_RUN_TOOL_TS: &str = r#"import { tool } from "@opencode-ai/plugin"
import { writeFileSync } from "fs"

export default tool({
  description: "Call this when the task is complete to submit the final result. REQUIRED at end of every run.",
  args: {
    status: tool.schema.enum(["completed", "failed", "blocked"]).describe("Outcome of the run"),
    pr_url: tool.schema.string().optional().describe("URL of the pull request, if created"),
    summary: tool.schema.string().optional().describe("Brief summary of what was done, what went wrong, or why blocked"),
    blocked_reason: tool.schema.string().optional().describe("If status is 'blocked', explain what input/approval is needed"),
    next_column: tool.schema.string().optional().describe("Suggested Kaneo column to move the task to (e.g. 'In Review', 'Blocked')"),
  },
  async execute(args) {
    writeFileSync(
      process.env.FINISH_ARTIFACT_PATH || `${process.env.HOME}/finish_artifact.json`,
      JSON.stringify(args, null, 2)
    )
    return { ok: true }
  },
})
"#;

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
