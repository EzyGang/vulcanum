mod launch;

use rand::Rng;
use vulcanum_shared::runtime::agent::{AgentRuntime, RunningSession};
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::IsolatedEnvironment;

use crate::runtime::client;
use crate::runtime::client::events;
use crate::runtime::client::health;
use crate::runtime::client::session;
use crate::runtime::runner::OpenCodeRunningSession;

const HEALTH_CHECK_TIMEOUT_SECS: u64 = 60;
const HEALTH_CHECK_INTERVAL_MS: u64 = 500;

pub struct OpenCodeServeRuntime;

impl OpenCodeServeRuntime {
    pub fn new() -> Self {
        Self
    }

    fn generate_password() -> String {
        let mut rng = rand::thread_rng();
        (0..32)
            .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
            .collect()
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

    async fn wait_for_health(client: &client::OpenCodeClient) -> Result<(), HarnessError> {
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_secs(HEALTH_CHECK_TIMEOUT_SECS);

        loop {
            match health::health_check(client).await {
                Ok(resp) if resp.healthy => return Ok(()),
                Ok(_) => {}
                Err(HarnessError::Http(_)) => {}
                Err(e) => return Err(e),
            }

            if std::time::Instant::now() >= deadline {
                return Err(HarnessError::ServerUnhealthy(format!(
                    "server not healthy after {HEALTH_CHECK_TIMEOUT_SECS}s"
                )));
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
        let password = Self::generate_password();
        let is_container = env.container_name.is_some();

        let (host_port, child_process) = if is_container {
            let (port, _cid) = launch::launch_container_server(env, &password).await?;
            (port, None)
        } else {
            let port = Self::discover_host_port()?;
            let child =
                launch::launch_host_server(&env.workdir, &env.env_vars, &password, port).await?;
            (port, Some(child))
        };

        let base_url = format!("http://127.0.0.1:{host_port}");
        let oc_client = client::OpenCodeClient::new(&base_url, "opencode", &password);

        Self::wait_for_health(&oc_client).await?;

        let sess = session::create_session(&oc_client, "vulcanum-run").await?;
        session::send_message_async(&oc_client, &sess.id, prompt).await?;

        let event_stream = events::connect_events(&oc_client).await?;

        let max_duration = env.limits.max_duration_secs;

        let runner = OpenCodeRunningSession::new(
            oc_client,
            sess.id,
            event_stream,
            max_duration,
            is_container,
            env.container_name.clone(),
            child_process,
        );

        Ok(Box::new(runner))
    }
}
