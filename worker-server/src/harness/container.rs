use std::collections::HashMap;
use std::path::Path;

use vulcanum_shared::constants::DEFAULT_IMAGE;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};
use vulcanum_shared::runtime::IsolationProvider;

use crate::harness::prepare;

pub struct DockerIsolation {
    pub(crate) image: String,
    pub(crate) runtime: &'static str,
}

impl DockerIsolation {
    pub fn new(runtime: &'static str) -> Self {
        let image = std::env::var("VULCANUM_IMAGE").unwrap_or_else(|_| DEFAULT_IMAGE.to_owned());
        Self { image, runtime }
    }

    #[allow(dead_code)]
    pub fn with_image(image: String, runtime: &'static str) -> Self {
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
        _limits: &ResourceLimits,
        agents_md: &str,
        opencode_config: &str,
        repo_url: &str,
    ) -> Result<IsolatedEnvironment, HarnessError> {
        self.ensure_image().await?;

        tokio::fs::create_dir_all(workdir)
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to create workdir: {e}")))?;

        prepare::write_env_files(workdir, agents_md, opencode_config).await?;

        if !repo_url.is_empty() {
            prepare::clone_repo(repo_url, &workdir.join("repo")).await?;
        }

        let container_name = prepare::container_name(workdir);

        let mut combined_env: HashMap<String, String> = secrets.clone();
        combined_env.insert("HOME".to_owned(), "/workdir/home".to_owned());

        Ok(IsolatedEnvironment {
            workdir: workdir.to_path_buf(),
            container_name: Some(container_name),
            secrets: secrets.clone(),
            env_vars: combined_env,
            runtime: Some(self.runtime),
            image: Some(self.image.clone()),
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
