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
    ) -> Result<HarnessResult, HarnessError> {
        let workdir = workdir.to_path_buf();
        let repo_url = repo_url.to_owned();

        let env = RunnerEnv {
            prompt,
            workdir: &workdir,
            limits,
            agents_md,
            spawn_error_msg: "opencode",
        };

        runner::run_opencode_in_env(
            env,
            || {
                let mut cmd = Command::new("opencode");
                cmd.arg("--prompt")
                    .arg(workdir.join("prompt.md"))
                    .current_dir(&workdir)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped());

                if !repo_url.is_empty() {
                    cmd.arg("--repo-url").arg(&repo_url);
                }

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
