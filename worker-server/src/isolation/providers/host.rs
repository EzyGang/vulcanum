use std::collections::HashMap;
use std::path::Path;

use tokio::fs;
use vulcanum_shared::api_types::{AgentBackend, AgentConfigPayload, JobRepo, WorkRunType};
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};

use crate::isolation::github_credentials;
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
        agent_backend: AgentBackend,
        agent_config: &AgentConfigPayload,
        repos: &[JobRepo],
    ) -> Result<IsolatedEnvironment, HarnessError> {
        fs::create_dir_all(workdir)
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to create workdir: {e}")))?;

        workspace::write_agent_files(workdir, agents_md, agent_backend, agent_config, work_type)
            .await?;

        let home_dir = workdir.join("home");
        let runtime_home = home_dir.to_string_lossy().to_string();
        let github_credentials = github_credentials::setup(
            workdir,
            github_credentials::token_from(secrets),
            &runtime_home,
        )
        .await?;
        let workspace_repos =
            workspace::prepare_repos(workdir, repos, &github_credentials.host_env, agent_backend)
                .await?;

        let sanitized_secrets = github_credentials::without_direct_token_env(secrets);
        let mut combined_env: HashMap<String, String> = env_vars.clone();
        for (key, value) in sanitized_secrets.clone() {
            combined_env.insert(key, value);
        }
        combined_env.insert("HOME".to_owned(), home_dir.to_string_lossy().to_string());
        combined_env.insert(
            "FINISH_ARTIFACT_PATH".to_owned(),
            home_dir
                .join("finish_artifact.json")
                .to_string_lossy()
                .to_string(),
        );
        match agent_backend {
            AgentBackend::OpenCode => {
                let config_dir = home_dir.join(".config").join("opencode");
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
            }
            AgentBackend::OmpRpc => {
                let config_home = home_dir.join(".omp");
                let state_home = home_dir.join(".local").join("state").join("omp");
                combined_env.insert(
                    "PI_CONFIG_HOME".to_owned(),
                    config_home.to_string_lossy().to_string(),
                );
                combined_env.insert(
                    "PI_DATA_HOME".to_owned(),
                    home_dir
                        .join(".local")
                        .join("share")
                        .join("omp")
                        .to_string_lossy()
                        .to_string(),
                );
                combined_env.insert(
                    "PI_STATE_HOME".to_owned(),
                    state_home.to_string_lossy().to_string(),
                );
                combined_env.insert(
                    "PI_SESSION_DIR".to_owned(),
                    config_home.join("sessions").to_string_lossy().to_string(),
                );
                combined_env.insert(
                    "PI_LOG_DIR".to_owned(),
                    state_home.join("logs").to_string_lossy().to_string(),
                );
                combined_env.insert(
                    "PI_TMPDIR".to_owned(),
                    workdir.join("tmp").to_string_lossy().to_string(),
                );
                combined_env.insert(
                    "PI_PERMISSION_DEFAULT".to_owned(),
                    "allow_always".to_owned(),
                );
            }
        }
        combined_env.extend(github_credentials.host_env);

        Ok(IsolatedEnvironment {
            workdir: workdir.to_path_buf(),
            workspace_dir: workdir.join("workspace"),
            repos: workspace_repos,
            container_name: None,
            secrets: sanitized_secrets,
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
                "refusing to remove unsafe host workdir"
            );
            return;
        }
        if let Err(e) = fs::remove_dir_all(&env.workdir).await {
            tracing::warn!(workdir = %env.workdir.display(), error = %e, "failed to remove host workdir");
        }
    }
}
