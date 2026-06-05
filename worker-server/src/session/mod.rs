mod running;

use std::process::Stdio;
use std::sync::Arc;

use tokio::process::Child;
use uuid::Uuid;

use vulcanum_shared::api_types::WireEvent;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::runtime::types::SessionStatus;

use crate::opencode::events::SseEventStream;
use crate::opencode::OpenCodeClient;

pub struct SessionConfig {
    pub client: OpenCodeClient,
    pub session_id: String,
    pub event_stream: SseEventStream,
    pub max_duration_secs: u64,
    pub container_name: Option<String>,
    pub server_process: Option<Child>,
    pub host_pid: Option<u32>,
    pub host_port: Option<u16>,
}

pub struct OpenCodeRunningSession {
    pub(crate) client: OpenCodeClient,
    pub(crate) session_id: String,
    pub(crate) event_stream: Option<SseEventStream>,
    pub(crate) status: SessionStatus,
    pub(crate) started_at: chrono::DateTime<chrono::Utc>,
    pub(crate) max_duration_secs: u64,
    pub(crate) container_name: Option<String>,
    pub(crate) server_process: Option<Child>,
    pub(crate) host_pid: Option<u32>,
    pub(crate) host_port: Option<u16>,
    pub(crate) api_client: Option<Arc<ApiClient>>,
    pub(crate) access_token: Option<String>,
    pub(crate) job_id: Option<Uuid>,
    pub(crate) event_sequence: u64,
}

const HIGH_LEVEL_EVENT_TYPES: &[&str] = &[
    "turn.started",
    "session.completed",
    "session.failed",
    "turn.failed",
];

impl OpenCodeRunningSession {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            client: config.client,
            session_id: config.session_id,
            event_stream: Some(config.event_stream),
            status: SessionStatus::Running,
            started_at: chrono::Utc::now(),
            max_duration_secs: config.max_duration_secs,
            container_name: config.container_name,
            server_process: config.server_process,
            host_pid: config.host_pid,
            host_port: config.host_port,
            api_client: None,
            access_token: None,
            job_id: None,
            event_sequence: 0,
        }
    }

    pub async fn kill_server(&mut self) {
        if let Some(ref mut child) = self.server_process {
            if let Some(pid) = child.id() {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &format!("-{pid}")])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
            let _ = child.wait().await;
        }
        remove_container(self.container_name.as_deref());
    }

    pub(crate) fn send_event(&mut self, event_type: &str, payload: serde_json::Value) {
        let (Some(client), Some(token), Some(job_id)) =
            (&self.api_client, &self.access_token, self.job_id)
        else {
            return;
        };

        self.event_sequence += 1;
        let wire = WireEvent {
            sequence: self.event_sequence,
            event_type: event_type.to_owned(),
            payload,
        };

        let c = Arc::clone(client);
        let t = token.clone();
        let jid = job_id;
        let events = vec![wire];
        tokio::spawn(async move {
            match c.append_events(jid, &events, &t).await {
                Ok(resp) => {
                    if resp.should_cancel {
                        tracing::warn!(
                            work_run_id = %jid,
                            "server requested cancel via event response"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        work_run_id = %jid,
                        error = %e,
                        "failed to send event to server"
                    );
                }
            }
        });
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
