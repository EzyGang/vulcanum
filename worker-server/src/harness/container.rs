use std::collections::HashMap;
use std::path::Path;

use vulcanum_shared::constants::DEFAULT_IMAGE;

use crate::harness::errors::HarnessError;
use crate::harness::runner::{self, RunnerEnv};
use crate::harness::{AgentHarness, HarnessResult, ResourceLimits};
use tokio::process::Command;

/// A generic container harness that spawns agent jobs inside Docker with a configurable runtime.
///
/// Used by both Kata Containers (--runtime=kata-runtime) and gVisor (--runtime=runsc)
/// to avoid duplicating the exact same spawn logic.
pub struct ContainerHarness {
    pub(crate) image: String,
    pub(crate) runtime: &'static str,
}

impl ContainerHarness {
    pub fn new(runtime: &'static str) -> Self {
        let image = std::env::var("VULCANUM_IMAGE").unwrap_or_else(|_| DEFAULT_IMAGE.to_owned());
        Self { image, runtime }
    }

    #[allow(dead_code)]
    pub fn with_image(image: String, runtime: &'static str) -> Self {
        Self { image, runtime }
    }

    pub(crate) async fn ensure_image(&self) {
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
        }
    }
}

impl AgentHarness for ContainerHarness {
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
        let runtime = self.runtime;
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
                    .arg(format!("--runtime={runtime}"))
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
                    .arg("/root/.local/bin/opencode")
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
