use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use tokio::process::Command;
use tokio::time::{sleep, timeout};

use crate::harness::errors::HarnessError;
use crate::harness::parse::{parse_pr_url, parse_token_usage};
use crate::harness::{AgentHarness, HarnessResult, ResourceLimits};

const TERM_GRACE_SECS: u64 = 5;

pub struct HostHarness;

impl HostHarness {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HostHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentHarness for HostHarness {
    async fn spawn(
        &self,
        prompt: &str,
        workdir: &Path,
        secrets: &HashMap<String, String>,
        limits: &ResourceLimits,
        repo_url: &str,
        agents_md: &str,
    ) -> Result<HarnessResult, HarnessError> {
        let prompt_path = workdir.join("prompt.md");
        tokio::fs::write(&prompt_path, prompt)
            .await
            .map_err(|e| HarnessError::OpenCodeCrash(format!("failed to write prompt: {e}")))?;

        if !agents_md.is_empty() {
            let agents_path = workdir.join("AGENTS.md");
            tokio::fs::write(&agents_path, agents_md)
                .await
                .map_err(|e| {
                    HarnessError::OpenCodeCrash(format!("failed to write AGENTS.md: {e}"))
                })?;
        }

        let mut cmd = Command::new("opencode");
        cmd.arg("--prompt")
            .arg(&prompt_path)
            .current_dir(workdir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        for (key, value) in secrets {
            cmd.env(key, value);
        }

        if !repo_url.is_empty() {
            cmd.arg("--repo-url").arg(repo_url);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| HarnessError::Install(format!("failed to spawn opencode: {e}")))?;

        let start = Instant::now();
        let max_duration = Duration::from_secs(limits.max_duration_secs);

        let exit_status = match timeout(max_duration, child.wait()).await {
            Ok(Ok(status)) => status,
            Ok(Err(e)) => {
                return Err(HarnessError::OpenCodeCrash(format!(
                    "opencode process error: {e}"
                )));
            }
            Err(_) => {
                let pid = child.id().unwrap_or(0);
                if pid > 0 {
                    let _ = Command::new("kill")
                        .arg("-TERM")
                        .arg(pid.to_string())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn();
                    sleep(Duration::from_secs(TERM_GRACE_SECS)).await;
                }
                let _ = child.kill().await;
                return Err(HarnessError::Timeout(limits.max_duration_secs));
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        let stdout = match child.stdout.take() {
            Some(mut out) => {
                let mut buf = String::new();
                match tokio::io::AsyncReadExt::read_to_string(&mut out, &mut buf).await {
                    Ok(_) => buf,
                    Err(e) => {
                        return Err(HarnessError::OutputParse(format!(
                            "failed to read stdout: {e}"
                        )));
                    }
                }
            }
            None => String::new(),
        };

        let pr_url = parse_pr_url(&stdout);
        let tokens_used = parse_token_usage(&stdout);

        Ok(HarnessResult {
            exit_code: exit_status.code().unwrap_or(-1),
            tokens_used,
            pr_url,
            duration_ms,
        })
    }
}
