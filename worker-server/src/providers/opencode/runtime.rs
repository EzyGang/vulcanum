use vulcanum_shared::runtime::agent::{AgentRuntime, RunningSession};
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::IsolatedEnvironment;

use super::api;
use super::events;
use super::runner::OpenCodeRunningSession;
use super::runner::SessionConfig;

const HEALTH_CHECK_TIMEOUT_SECS: u64 = 180;
const HEALTH_CHECK_INTERVAL_MS: u64 = 3000;
const HEALTH_CHECK_REQUEST_TIMEOUT_SECS: u64 = 5;

pub struct OpenCodeServeRuntime;

impl OpenCodeServeRuntime {
    pub fn new() -> Self {
        Self
    }

    fn discover_host_port() -> Result<u16, HarnessError> {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .map_err(|e| HarnessError::ServerLaunch(format!("bind failed: {e}")))?;
        let port = listener
            .local_addr()
            .map_err(|e| HarnessError::ServerLaunch(format!("local_addr failed: {e}")))?
            .port();
        drop(listener);
        Ok(port)
    }

    async fn wait_for_health(
        client: &super::OpenCodeClient,
        child: &mut Option<tokio::process::Child>,
        container_name: Option<&str>,
    ) -> Result<(), HarnessError> {
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_secs(HEALTH_CHECK_TIMEOUT_SECS);

        loop {
            if let Some(ref mut c) = child {
                match c.try_wait() {
                    Ok(Some(status)) => {
                        return Err(HarnessError::ServerLaunch(format!(
                            "opencode process exited prematurely with {status}"
                        )));
                    }
                    Ok(None) => (),
                    Err(e) => {
                        return Err(HarnessError::ServerLaunch(format!(
                            "failed to check process status: {e}"
                        )));
                    }
                }
            }

            if std::time::Instant::now() >= deadline {
                let mut msg = format!("server not healthy after {HEALTH_CHECK_TIMEOUT_SECS}s");
                if let Some(name) = container_name {
                    if let Ok(output) = tokio::process::Command::new("docker")
                        .args(["inspect", "--format={{.State.Status}}", name])
                        .output()
                        .await
                    {
                        let status = String::from_utf8_lossy(&output.stdout).trim().to_owned();
                        msg.push_str(&format!("; container status: {status}"));
                    }
                }
                tracing::warn!("{msg}");
                return Err(HarnessError::ServerUnhealthy(msg));
            }

            let health_result = tokio::time::timeout(
                std::time::Duration::from_secs(HEALTH_CHECK_REQUEST_TIMEOUT_SECS),
                super::health::health_check(client),
            )
            .await;

            match health_result {
                Ok(Ok(resp)) if resp.healthy => return Ok(()),
                Ok(Ok(_)) => {}
                Ok(Err(e)) => {
                    tracing::debug!(error = %e, "health check failed");
                }
                Err(_) => {
                    tracing::debug!("health check request timed out");
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(HEALTH_CHECK_INTERVAL_MS)).await;
        }
    }
}

impl AgentRuntime for OpenCodeServeRuntime {
    async fn execute(
        &self,
        prompt: &str,
        env: &IsolatedEnvironment,
        repo_url: &str,
    ) -> Result<Box<dyn RunningSession>, HarnessError> {
        let is_container = env.container_name.is_some();
        let has_repo = !repo_url.is_empty();

        let (host_port, mut child_process) = if is_container {
            let repo_dir = if has_repo { "/workdir/repo" } else { "" };
            let (port, _cid) = super::spawn::launch_container_server(env, repo_dir).await?;
            (port, None)
        } else {
            let port = Self::discover_host_port()?;
            let repo_dir = has_repo.then(|| env.workdir.join("repo"));
            let child = super::spawn::launch_host_server(
                &env.workdir,
                &env.env_vars,
                port,
                repo_dir.as_deref(),
            )
            .await?;
            (port, Some(child))
        };

        let base_url = format!("http://127.0.0.1:{host_port}");
        let oc_client = super::OpenCodeClient::new(&base_url);

        Self::wait_for_health(
            &oc_client,
            &mut child_process,
            env.container_name.as_deref(),
        )
        .await?;
        tracing::debug!(host_port, "opencode server healthy");

        let event_stream = events::connect_events(&oc_client).await?;
        tracing::debug!("event stream connected");

        let sess = api::create_session(&oc_client, "vulcanum-run").await?;
        tracing::debug!(session_id = %sess.id, "session created");
        api::send_message_async(&oc_client, &sess.id, prompt).await?;
        tracing::debug!(session_id = %sess.id, prompt_len = prompt.len(), "prompt submitted");

        let max_duration = env.limits.max_duration_secs;

        let runner_session_id = sess.id.clone();

        let (host_pid, host_port) = match &child_process {
            Some(child) => (child.id(), Some(host_port)),
            None => (None, None),
        };

        let runner = OpenCodeRunningSession::new(SessionConfig {
            client: oc_client,
            session_id: sess.id,
            event_stream,
            max_duration_secs: max_duration,
            container_name: env.container_name.clone(),
            server_process: child_process,
            host_pid,
            host_port,
        });

        tracing::info!(
            session_id = %runner_session_id,
            max_duration_secs = max_duration,
            "session runner ready, starting event loop"
        );

        Ok(Box::new(runner))
    }
}
