use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use regex::Regex;
use tokio::process::Command;
use tokio::time::timeout;

use crate::harness::errors::HarnessError;
use crate::harness::{AgentHarness, HarnessResult, ResourceLimits};

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
    ) -> Result<HarnessResult, HarnessError> {
        let prompt_path = workdir.join("prompt.md");
        tokio::fs::write(&prompt_path, prompt)
            .await
            .map_err(|e| HarnessError::OpenCodeCrash(format!("failed to write prompt: {e}")))?;

        let mut cmd = Command::new("opencode");
        cmd.arg("--prompt")
            .arg(&prompt_path)
            .current_dir(workdir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        for (key, value) in secrets {
            cmd.env(key, value);
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
                let _ = child.kill().await;
                return Err(HarnessError::VmTimeout(limits.max_duration_secs));
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

pub(super) fn parse_pr_url(text: &str) -> Option<String> {
    static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"https://github\.com/[^/\s]+/[^/\s]+/pull/\d+").unwrap()
    });
    RE.find(text).map(|m| m.as_str().to_owned())
}

pub(super) fn parse_token_usage(text: &str) -> u64 {
    static RE: std::sync::LazyLock<Regex> =
        std::sync::LazyLock::new(|| Regex::new(r"Tokens used:\s*(\d+)").unwrap());
    RE.captures(text)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0)
}
