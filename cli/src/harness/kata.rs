use std::collections::HashMap;
use std::path::Path;

use tokio::process::Command;

use crate::harness::errors::HarnessError;
use crate::harness::runner::{self, RunnerEnv};
use crate::harness::{AgentHarness, HarnessResult, ResourceLimits};

pub(crate) const DEFAULT_KATA_IMAGE: &str = "ghcr.io/vulcanum/agent:latest";

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
        repo_url: &str,
        agents_md: &str,
    ) -> Result<HarnessResult, HarnessError> {
        let container_name = format!(
            "vulcanum-{}",
            workdir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("job")
        );
        let workdir_for_cmd = workdir.to_path_buf();
        let repo_url_for_cmd = repo_url.to_owned();
        let image = self.image.clone();
        let limits_val = *limits;

        let cleanup_name = container_name.clone();
        let workdir_ref = workdir_for_cmd.clone();

        let env = RunnerEnv {
            prompt,
            workdir: &workdir_ref,
            limits,
            agents_md,
            spawn_error_msg: "docker",
        };

        runner::run_opencode_in_env(
            env,
            move || {
                let mut cmd = Command::new("docker");
                cmd.arg("run")
                    .arg("--runtime=kata-runtimes")
                    .arg("--rm")
                    .arg("--name")
                    .arg(&container_name)
                    .arg("-v")
                    .arg(format!("{}:/workdir", workdir_for_cmd.display()))
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

                if !repo_url_for_cmd.is_empty() {
                    cmd.arg("--repo-url").arg(&repo_url_for_cmd);
                }

                Ok(cmd)
            },
            Some(&cleanup_name),
        )
        .await
    }
}
