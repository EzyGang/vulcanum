use std::collections::HashMap;
use std::path::Path;

use tokio::fs;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};

use crate::harness::prepare;

pub struct HostIsolation;

impl HostIsolation {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HostIsolation {
    fn default() -> Self {
        Self::new()
    }
}

impl IsolationProvider for HostIsolation {
    async fn prepare(
        &self,
        workdir: &Path,
        secrets: &HashMap<String, String>,
        env_vars: &HashMap<String, String>,
        limits: &ResourceLimits,
        agents_md: &str,
        opencode_config: &str,
        repo_url: &str,
    ) -> Result<IsolatedEnvironment, HarnessError> {
        fs::create_dir_all(workdir)
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to create workdir: {e}")))?;

        prepare::write_env_files(workdir, agents_md, opencode_config).await?;
        prepare::write_finish_run_tool(workdir).await?;

        if !repo_url.is_empty() {
            prepare::clone_repo(repo_url, &workdir.join("repo")).await?;
        }

        let mut combined_env: HashMap<String, String> = env_vars.clone();
        for (k, v) in secrets {
            combined_env.insert(k.clone(), v.clone());
        }
        combined_env.insert(
            "HOME".to_owned(),
            workdir.join("home").to_string_lossy().to_string(),
        );

        Ok(IsolatedEnvironment {
            workdir: workdir.to_path_buf(),
            container_name: None,
            secrets: secrets.clone(),
            env_vars: combined_env,
            runtime: None,
            image: None,
            server_host_port: None,
            limits: limits.clone(),
        })
    }

    async fn cleanup(&self, _env: &IsolatedEnvironment) {}
}
