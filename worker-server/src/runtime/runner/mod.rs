mod running;

use std::process::Stdio;

use tokio::process::Child;
use vulcanum_shared::runtime::types::SessionStatus;

use crate::runtime::client::events::SseEventStream;
use crate::runtime::client::OpenCodeClient;

pub struct OpenCodeRunningSession {
    pub(crate) client: OpenCodeClient,
    pub(crate) session_id: String,
    pub(crate) event_stream: Option<SseEventStream>,
    pub(crate) status: SessionStatus,
    pub(crate) started_at: chrono::DateTime<chrono::Utc>,
    pub(crate) max_duration_secs: u64,
    pub(crate) is_container: bool,
    pub(crate) container_name: Option<String>,
    pub(crate) server_process: Option<Child>,
}

impl OpenCodeRunningSession {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        client: OpenCodeClient,
        session_id: String,
        event_stream: SseEventStream,
        max_duration_secs: u64,
        is_container: bool,
        container_name: Option<String>,
        server_process: Option<Child>,
    ) -> Self {
        Self {
            client,
            session_id,
            event_stream: Some(event_stream),
            status: SessionStatus::Running,
            started_at: chrono::Utc::now(),
            max_duration_secs,
            is_container,
            container_name,
            server_process,
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub async fn kill_server(&mut self) {
        if let Some(ref mut child) = self.server_process {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        remove_container(self.container_name.as_deref());
    }
}

pub(crate) fn remove_container(name: Option<&str>) {
    let Some(name) = name else {
        return;
    };
    let _ = std::process::Command::new("docker")
        .args(["rm", "-f", name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
