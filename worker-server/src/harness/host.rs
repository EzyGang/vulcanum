use std::collections::HashMap;
use std::path::Path;

use tokio::process::Command;

use crate::harness::errors::HarnessError;
use crate::harness::runner::{self, RunnerEnv};
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
        repo_url: &str,
        agents_md: &str,
        opencode_config: &str,
    ) -> Result<HarnessResult, HarnessError> {
        let workdir = workdir.to_path_buf();

        let env = RunnerEnv {
            prompt,
            workdir: &workdir,
            limits,
            agents_md,
            opencode_config,
            repo_url,
            spawn_error_msg: "opencode",
        };

        runner::run_opencode_in_env(
            env,
            || {
                let mut cmd = Command::new("opencode");
                let repo_dir = workdir.join("repo");
                let target_dir = if repo_dir.exists() {
                    &repo_dir
                } else {
                    &workdir
                };

                cmd.arg("run")
                    .arg("--dir")
                    .arg(target_dir)
                    .arg("--dangerously-skip-permissions")
                    .env("HOME", workdir.join("home"))
                    .current_dir(&workdir)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .stdin(std::process::Stdio::piped());

                for (key, value) in secrets {
                    cmd.env(key, value);
                }

                Ok(cmd)
            },
            None,
        )
        .await
    }
}
