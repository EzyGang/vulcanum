use std::collections::HashMap;
use std::path::Path;

use tokio::process::Command;

use crate::harness::errors::HarnessError;
use crate::harness::runner::{self, RunnerEnv};
use crate::harness::{AgentHarness, HarnessResult, ResourceLimits};

pub(crate) const DEFAULT_KATA_IMAGE: &str = "ghcr.io/ezygang/vulcanum/agent:latest";

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

    async fn ensure_image(&self) {
        let output = match Command::new("docker")
            .args(["images", "-q", &self.image])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()
            .await
        {
            Ok(o) => o,
            Err(e) => {
                tracing::warn!("failed to check docker images: {e}");
                return;
            }
        };

        if !String::from_utf8_lossy(&output.stdout).trim().is_empty() {
            return;
        }

        tracing::info!("agent image missing, pulling {}...", &self.image);

        let status = match Command::new("docker")
            .args(["pull", &self.image])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
        {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("failed to pull agent image: {e}");
                return;
            }
        };

        if !status.success() {
            tracing::warn!(
                "docker pull '{}' failed — will retry on next job",
                &self.image
            );
        } else {
            tracing::info!("agent image pulled successfully");
        }
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
        repo_url: &str,
        agents_md: &str,
    ) -> Result<HarnessResult, HarnessError> {
        self.ensure_image().await;

        let container_name = format!(
            "vulcanum-{}",
            workdir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("job")
        );
        let workdir_for_cmd = workdir.to_path_buf();
        let image = self.image.clone();
        let limits_val = *limits;

        let cleanup_name = container_name.clone();
        let workdir_ref = workdir_for_cmd.clone();

        let env = RunnerEnv {
            prompt,
            workdir: &workdir_ref,
            limits,
            agents_md,
            repo_url,
            spawn_error_msg: "docker",
        };

        runner::run_opencode_in_env(
            env,
            move || {
                let mut cmd = Command::new("docker");
                let repo_path = workdir_for_cmd.join("repo");

                cmd.arg("run")
                    .arg("--runtime=kata-runtimes")
                    .arg("--rm")
                    .arg("--name")
                    .arg(&container_name)
                    .arg("-v")
                    .arg(format!("{}:/workdir", workdir_for_cmd.display()))
                    .arg("-e")
                    .arg("HOME=/workdir/home")
                    .arg(format!("--cpus={}", limits_val.vcpu_count))
                    .arg(format!("--memory={}m", limits_val.memory_mib))
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::inherit());

                for (key, value) in secrets {
                    cmd.arg("-e").arg(format!("{key}={value}"));
                }

                cmd.arg(&image)
                    .arg("opencode")
                    .arg("--prompt")
                    .arg("/workdir/prompt.md");

                if repo_path.exists() {
                    cmd.arg("--dir").arg("/workdir/repo");
                }

                Ok(cmd)
            },
            Some(&cleanup_name),
        )
        .await
    }
}
