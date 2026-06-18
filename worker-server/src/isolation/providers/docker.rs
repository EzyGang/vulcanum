use std::collections::HashMap;
use std::path::Path;

use vulcanum_shared::api_types::JobRepo;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};

use crate::isolation::workspace;

pub struct DockerIsolation {
    pub(crate) image: String,
    pub(crate) runtime: Option<&'static str>,
}

impl DockerIsolation {
    pub fn new(runtime: Option<&'static str>, image: String) -> Self {
        Self { image, runtime }
    }

    #[allow(dead_code)]
    pub fn with_image(image: String, runtime: Option<&'static str>) -> Self {
        Self { image, runtime }
    }

    pub(crate) async fn ensure_image(&self) -> Result<(), HarnessError> {
        let status = tokio::process::Command::new("docker")
            .args(["pull", &self.image])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
            .map_err(|e| HarnessError::Install(format!("failed to pull agent image: {e}")))?;

        if !status.success() {
            return Err(HarnessError::Install(format!(
                "docker pull '{}' failed",
                &self.image
            )));
        }

        Ok(())
    }
}

impl IsolationProvider for DockerIsolation {
    async fn prepare(
        &self,
        workdir: &Path,
        secrets: &HashMap<String, String>,
        _env_vars: &HashMap<String, String>,
        limits: &ResourceLimits,
        agents_md: &str,
        generated_opencode_config: &str,
        repos: &[JobRepo],
    ) -> Result<IsolatedEnvironment, HarnessError> {
        self.ensure_image().await?;

        tokio::fs::create_dir_all(workdir)
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to create workdir: {e}")))?;

        workspace::write_env_files(workdir, agents_md, generated_opencode_config).await?;
        workspace::write_finish_run_tool(workdir).await?;

        let token = secrets.get("GITHUB_TOKEN").map(|s| s.as_str());
        let workspace_repos = workspace::prepare_repos(workdir, repos, token).await?;

        let container_name = workspace::container_name(workdir);

        let mut combined_env: HashMap<String, String> = secrets.clone();
        combined_env.insert("HOME".to_owned(), "/workdir/home".to_owned());
        combined_env.insert(
            "OPENCODE_CONFIG".to_owned(),
            "/workdir/home/.config/opencode/opencode.json".to_owned(),
        );
        combined_env.insert(
            "OPENCODE_CONFIG_DIR".to_owned(),
            "/workdir/home/.config/opencode".to_owned(),
        );

        Ok(IsolatedEnvironment {
            workdir: workdir.to_path_buf(),
            workspace_dir: workdir.join("workspace"),
            repos: workspace_repos,
            container_name: Some(container_name),
            secrets: secrets.clone(),
            env_vars: combined_env,
            runtime: self.runtime,
            image: Some(self.image.clone()),
            server_host_port: None,
            limits: limits.clone(),
        })
    }

    async fn cleanup(&self, env: &IsolatedEnvironment) {
        let Some(name) = &env.container_name else {
            return;
        };

        let result = tokio::process::Command::new("docker")
            .args(["rm", "-f", name])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        if let Ok(mut child) = result {
            let _ = child.wait().await;
        }
    }
}
