mod launch;

use vulcanum_shared::runtime::agent::{AgentRuntime, RunningSession};
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::IsolatedEnvironment;

use crate::runtime::client;
use crate::runtime::client::events;
use crate::runtime::client::health;
use crate::runtime::client::health::HealthResponse;
use crate::runtime::client::session;
use crate::runtime::runner::{OpenCodeRunningSession, SessionConfig};

const HEALTH_CHECK_TIMEOUT_SECS: u64 = 180;
const HEALTH_CHECK_INTERVAL_MS: u64 = 500;

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
        client: &client::OpenCodeClient,
        child: &mut Option<tokio::process::Child>,
        container_name: Option<&str>,
    ) -> Result<(), HarnessError> {
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_secs(HEALTH_CHECK_TIMEOUT_SECS);
        let mut last_health: Option<HealthResponse> = None;
        let mut last_error: Option<String> = None;

        loop {
            match health::health_check(client).await {
                Ok(resp) if resp.healthy => return Ok(()),
                Ok(resp) => {
                    last_health = Some(resp);
                }
                Err(HarnessError::Http(ref msg)) => {
                    tracing::debug!(error = %msg, "health check connection error, retrying");
                    last_error = Some(msg.clone());
                }
                Err(HarnessError::ServerUnhealthy(ref msg)) => {
                    tracing::debug!(error = %msg, "health check server not ready, retrying");
                    last_error = Some(msg.clone());
                }
                Err(e) => return Err(e),
            }

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
                let waited = HEALTH_CHECK_TIMEOUT_SECS;
                let mut msg = format!("server not healthy after {waited}s");
                if let Some(ref resp) = last_health {
                    msg.push_str(&format!(
                        "; last health: healthy={}, version={}",
                        resp.healthy, resp.version
                    ));
                }
                if let Some(ref err) = last_error {
                    msg.push_str(&format!("; last error: {err}"));
                }
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
                tracing::warn!(error = %msg, "health check timed out");
                return Err(HarnessError::ServerUnhealthy(msg));
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
        _repo_url: &str,
        _agents_md: &str,
        _opencode_config: &str,
    ) -> Result<Box<dyn RunningSession>, HarnessError> {
        let is_container = env.container_name.is_some();

        let (host_port, mut child_process) = if is_container {
            let (port, _cid) = launch::launch_container_server(env).await?;
            (port, None)
        } else {
            let port = Self::discover_host_port()?;
            let child = launch::launch_host_server(&env.workdir, &env.env_vars, port).await?;
            (port, Some(child))
        };

        let base_url = format!("http://127.0.0.1:{host_port}");
        let oc_client = client::OpenCodeClient::new(&base_url);

        Self::wait_for_health(
            &oc_client,
            &mut child_process,
            env.container_name.as_deref(),
        )
        .await?;
        tracing::debug!(host_port, "opencode server healthy");

        let sess = session::create_session(&oc_client, "vulcanum-run").await?;
        tracing::debug!(session_id = %sess.id, "session created");
        session::send_message_async(&oc_client, &sess.id, prompt).await?;
        tracing::debug!(session_id = %sess.id, prompt_len = prompt.len(), "prompt submitted");

        let event_stream = events::connect_events(&oc_client).await?;
        tracing::debug!(session_id = %sess.id, "event stream connected");

        let max_duration = env.limits.max_duration_secs;

        let runner = OpenCodeRunningSession::new(SessionConfig {
            client: oc_client,
            session_id: sess.id,
            event_stream,
            max_duration_secs: max_duration,
            is_container,
            container_name: env.container_name.clone(),
            server_process: child_process,
        });

        tracing::info!(
            session_id = %runner.session_id(),
            max_duration_secs = max_duration,
            "session runner ready, starting event loop"
        );

        Ok(Box::new(runner))
    }
}
