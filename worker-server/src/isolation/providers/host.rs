use std::collections::HashMap;
use std::path::Path;

use tokio::fs;
use vulcanum_shared::api_types::{JobRepo, WorkRunType};
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};

use crate::isolation::workspace;

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

fn is_safe_workdir(path: &Path) -> bool {
    let temp = std::env::temp_dir();
    path.starts_with(&temp)
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("vulcanum-work-"))
}

impl IsolationProvider for HostIsolation {
    async fn prepare(
        &self,
        workdir: &Path,
        secrets: &HashMap<String, String>,
        env_vars: &HashMap<String, String>,
        limits: &ResourceLimits,
        work_type: WorkRunType,
        agents_md: &str,
        generated_opencode_config: &str,
        repos: &[JobRepo],
    ) -> Result<IsolatedEnvironment, HarnessError> {
        fs::create_dir_all(workdir)
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to create workdir: {e}")))?;

        workspace::write_env_files(workdir, agents_md, generated_opencode_config).await?;
        workspace::write_finish_run_tool(workdir, work_type).await?;

        let token = secrets.get("GITHUB_TOKEN").map(|s| s.as_str());
        let workspace_repos = workspace::prepare_repos(workdir, repos, token).await?;

        let mut combined_env: HashMap<String, String> = env_vars.clone();
        for (k, v) in secrets {
            combined_env.insert(k.clone(), v.clone());
        }
        let home_dir = workdir.join("home");
        let config_dir = home_dir.join(".config").join("opencode");
        combined_env.insert("HOME".to_owned(), home_dir.to_string_lossy().to_string());
        combined_env.insert(
            "OPENCODE_CONFIG".to_owned(),
            config_dir
                .join("opencode.json")
                .to_string_lossy()
                .to_string(),
        );
        combined_env.insert(
            "OPENCODE_CONFIG_DIR".to_owned(),
            config_dir.to_string_lossy().to_string(),
        );

        Ok(IsolatedEnvironment {
            workdir: workdir.to_path_buf(),
            workspace_dir: workdir.join("workspace"),
            repos: workspace_repos,
            container_name: None,
            secrets: secrets.clone(),
            env_vars: combined_env,
            runtime: None,
            image: None,
            server_host_port: None,
            limits: limits.clone(),
        })
    }

    async fn cleanup(&self, env: &IsolatedEnvironment) {
        if !is_safe_workdir(&env.workdir) {
            tracing::warn!(
                workdir = %env.workdir.display(),
                "refusing to delete unsafe workdir"
            );
            return;
        }
        if let Err(e) = fs::remove_dir_all(&env.workdir).await {
            tracing::warn!(
                workdir = %env.workdir.display(),
                error = %e,
                "cleanup failed"
            );
        }
    }
}
