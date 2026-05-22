use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use tokio::process::Command;
use tokio::time::{sleep, timeout};

use crate::harness::errors::HarnessError;
use crate::harness::parse::{parse_pr_url, parse_token_usage};
use crate::harness::{HarnessResult, ResourceLimits};

const TERM_GRACE_SECS: u64 = 5;

pub(super) struct RunnerEnv<'a> {
    pub prompt: &'a str,
    pub workdir: &'a Path,
    pub secrets: &'a HashMap<String, String>,
    pub limits: &'a ResourceLimits,
    pub agents_md: &'a str,
    pub spawn_error_msg: &'a str,
}

pub(super) async fn run_opencode_in_env(
    env: RunnerEnv<'_>,
    build_cmd: impl FnOnce() -> std::io::Result<Command>,
    container_name: Option<&str>,
) -> Result<HarnessResult, HarnessError> {
    let prompt_path = env.workdir.join("prompt.md");
    tokio::fs::write(&prompt_path, env.prompt)
        .await
        .map_err(|e| HarnessError::OpenCodeCrash(format!("failed to write prompt: {e}")))?;

    if !env.agents_md.is_empty() {
        let agents_path = env.workdir.join("AGENTS.md");
        tokio::fs::write(&agents_path, env.agents_md)
            .await
            .map_err(|e| HarnessError::OpenCodeCrash(format!("failed to write AGENTS.md: {e}")))?;
    }

    let mut cmd = build_cmd().map_err(|e| {
        HarnessError::Install(format!("failed to build {}: {e}", env.spawn_error_msg))
    })?;

    for (key, value) in env.secrets {
        cmd.env(key, value);
    }

    let mut child = cmd.spawn().map_err(|e| {
        HarnessError::Install(format!("failed to spawn {}: {e}", env.spawn_error_msg))
    })?;

    let start = Instant::now();
    let max_duration = Duration::from_secs(env.limits.max_duration_secs);

    let exit_status = match timeout(max_duration, child.wait()).await {
        Ok(Ok(status)) => status,
        Ok(Err(e)) => {
            cleanup_container(container_name).await;
            return Err(HarnessError::OpenCodeCrash(format!(
                "{} process error: {e}",
                env.spawn_error_msg
            )));
        }
        Err(_) => {
            kill_child_with_grace(&mut child).await;
            cleanup_container(container_name).await;
            return Err(HarnessError::Timeout(env.limits.max_duration_secs));
        }
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    let stdout = match child.stdout.take() {
        Some(mut out) => {
            let mut buf = String::new();
            match tokio::io::AsyncReadExt::read_to_string(&mut out, &mut buf).await {
                Ok(_) => buf,
                Err(e) => {
                    cleanup_container(container_name).await;
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

async fn kill_child_with_grace(child: &mut tokio::process::Child) {
    let pid = child.id().unwrap_or(0);
    if pid > 0 {
        let _ = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
        for _ in 0..TERM_GRACE_SECS {
            match child.try_wait() {
                Ok(Some(_)) => break,
                _ => sleep(Duration::from_secs(1)).await,
            }
        }
    }
    let _ = child.kill().await;
}

async fn cleanup_container(name: Option<&str>) {
    let Some(name) = name else {
        return;
    };
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
