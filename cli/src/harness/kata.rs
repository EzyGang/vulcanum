use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use tokio::process::Command;
use tokio::time::timeout;

use crate::harness::errors::HarnessError;
use crate::harness::parse::{parse_pr_url, parse_token_usage};
use crate::harness::{AgentHarness, HarnessResult, ResourceLimits};

const DEFAULT_KATA_IMAGE: &str = "ghcr.io/vulcanum/agent:latest";

pub struct KataHarness {
    pub(crate) image: String,
}

impl KataHarness {
    pub fn new() -> Self {
        let image = std::env::var("KATA_IMAGE").unwrap_or_else(|_| DEFAULT_KATA_IMAGE.to_owned());
        Self { image }
    }

    #[allow(dead_code)]
    pub fn with_image(image: String) -> Self {
        Self { image }
    }
}

impl Default for KataHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentHarness for KataHarness {
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

        let container_name = format!(
            "vulcanum-{}",
            workdir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("job")
        );

        let mut cmd = Command::new("docker");
        cmd.arg("run")
            .arg("--runtime=kata-runtimes")
            .arg("--rm")
            .arg("--name")
            .arg(&container_name)
            .arg("-v")
            .arg(format!("{}:/workdir", workdir.display()))
            .arg(format!("--cpus={}", limits.vcpu_count))
            .arg(format!("--memory={}m", limits.memory_mib))
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        for (key, value) in secrets {
            cmd.arg("-e").arg(format!("{key}={value}"));
        }

        cmd.arg(&self.image)
            .arg("opencode")
            .arg("--prompt")
            .arg("/workdir/prompt.md");

        let mut child = cmd
            .spawn()
            .map_err(|e| HarnessError::Install(format!("failed to spawn docker: {e}")))?;

        let start = Instant::now();
        let max_duration = Duration::from_secs(limits.max_duration_secs);

        let exit_status = match timeout(max_duration, child.wait()).await {
            Ok(Ok(status)) => status,
            Ok(Err(e)) => {
                let _ = cleanup_container(&container_name).await;
                return Err(HarnessError::OpenCodeCrash(format!(
                    "docker process error: {e}"
                )));
            }
            Err(_) => {
                let _ = child.kill().await;
                let _ = cleanup_container(&container_name).await;
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
                        let _ = cleanup_container(&container_name).await;
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

        let _ = cleanup_container(&container_name).await;

        Ok(HarnessResult {
            exit_code: exit_status.code().unwrap_or(-1),
            tokens_used,
            pr_url,
            duration_ms,
        })
    }
}

async fn cleanup_container(name: &str) {
    let result = Command::new("docker")
        .arg("rm")
        .arg("-f")
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    if let Ok(mut child) = result {
        let _ = child.wait().await;
    }
}
