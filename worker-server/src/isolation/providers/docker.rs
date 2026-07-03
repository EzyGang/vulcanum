use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;

use tokio::fs;

use vulcanum_shared::api_types::{AgentBackend, AgentConfigPayload, JobRepo, WorkRunType};
use vulcanum_shared::runtime::docker::retry_docker_pull;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};

use crate::isolation::github_credentials;
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
        retry_docker_pull(&self.image, || docker_pull_once(&self.image))
            .await
            .map_err(|e| HarnessError::Install(e.to_string()))
    }
}

async fn docker_pull_once(image: &str) -> Result<(), String> {
    let status = tokio::process::Command::new("docker")
        .args(["pull", image])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| format!("failed to pull agent image: {e}"))?;

    if status.success() {
        return Ok(());
    }

    Err(format!("exited with status {status}"))
}

impl IsolationProvider for DockerIsolation {
    async fn prepare(
        &self,
        workdir: &Path,
        secrets: &HashMap<String, String>,
        _env_vars: &HashMap<String, String>,
        limits: &ResourceLimits,
        work_type: WorkRunType,
        agents_md: &str,
        agent_backend: AgentBackend,
        agent_config: &AgentConfigPayload,
        repos: &[JobRepo],
    ) -> Result<IsolatedEnvironment, HarnessError> {
        self.ensure_image().await?;

        tokio::fs::create_dir_all(workdir)
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to create workdir: {e}")))?;

        workspace::write_agent_files(workdir, agents_md, agent_backend, agent_config, work_type)
            .await?;

        let github_credentials = github_credentials::setup(
            workdir,
            github_credentials::token_from(secrets),
            "/workdir/home",
        )
        .await?;
        let workspace_repos =
            workspace::prepare_repos(workdir, repos, &github_credentials.host_env, agent_backend)
                .await?;

        let container_name = workspace::container_name(workdir);

        let sanitized_secrets = github_credentials::without_direct_token_env(secrets);
        let mut combined_env: HashMap<String, String> = sanitized_secrets.clone();
        combined_env.insert("HOME".to_owned(), "/workdir/home".to_owned());
        combined_env.insert(
            "FINISH_ARTIFACT_PATH".to_owned(),
            "/workdir/home/finish_artifact.json".to_owned(),
        );
        match agent_backend {
            AgentBackend::OpenCode => {
                combined_env.insert(
                    "OPENCODE_CONFIG".to_owned(),
                    "/workdir/home/.config/opencode/opencode.json".to_owned(),
                );
                combined_env.insert(
                    "OPENCODE_CONFIG_DIR".to_owned(),
                    "/workdir/home/.config/opencode".to_owned(),
                );
            }
            AgentBackend::OmpRpc => {
                combined_env.extend(workspace::omp_environment_vars(
                    "/workdir/home",
                    "/workdir/tmp",
                ));
            }
        }
        combined_env.extend(github_credentials.runtime_env);

        Ok(IsolatedEnvironment {
            workdir: workdir.to_path_buf(),
            workspace_dir: workdir.join("workspace"),
            repos: workspace_repos,
            container_name: Some(container_name),
            secrets: sanitized_secrets,
            env_vars: combined_env,
            runtime: self.runtime,
            image: Some(self.image.clone()),
            server_host_port: None,
            limits: limits.clone(),
        })
    }

    async fn cleanup(&self, env: &IsolatedEnvironment) {
        if let Some(name) = &env.container_name {
            let result = tokio::process::Command::new("docker")
                .args(["rm", "-f", name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            if let Ok(mut child) = result {
                let _ = child.wait().await;
            }
        }

        cleanup_docker_workdir(&env.workdir, &self.image, self.runtime).await;
    }
}

pub(crate) async fn cleanup_docker_workdir(
    workdir: &Path,
    image: &str,
    runtime: Option<&'static str>,
) {
    if !is_safe_workdir(workdir) {
        tracing::warn!(
            workdir = %workdir.display(),
            "refusing to remove unsafe docker workdir"
        );
        return;
    }

    match fs::remove_dir_all(workdir).await {
        Ok(()) => return,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
        Err(e) => {
            tracing::warn!(
                workdir = %workdir.display(),
                error = %e,
                "host docker workdir removal failed, trying container cleanup"
            );
        }
    }

    if let Err(e) = remove_workdir_contents_with_container(workdir, image, runtime).await {
        tracing::warn!(
            workdir = %workdir.display(),
            error = %e,
            "container docker workdir cleanup failed"
        );
    }

    match fs::remove_dir_all(workdir).await {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => {
            tracing::warn!(workdir = %workdir.display(), error = %e, "failed to remove docker workdir");
        }
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

async fn remove_workdir_contents_with_container(
    workdir: &Path,
    image: &str,
    runtime: Option<&'static str>,
) -> Result<(), String> {
    let mut command = tokio::process::Command::new("docker");
    command.args(["run", "--rm"]);
    if let Some(runtime) = runtime {
        command.args(["--runtime", runtime]);
    }
    command
        .arg("-v")
        .arg(format!("{}:/workdir", workdir.display()))
        .args(["--workdir", "/workdir"])
        .arg(image)
        .args([
            "sh",
            "-c",
            "rm -rf /workdir/* /workdir/.[!.]* /workdir/..?*",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let status = command
        .status()
        .await
        .map_err(|e| format!("failed to start cleanup container: {e}"))?;
    if status.success() {
        return Ok(());
    }

    Err(format!("cleanup container exited with status {status}"))
}
