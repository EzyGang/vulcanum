use tokio::process::Child;
use vulcanum_shared::runtime::types::SessionStatus;

use super::events::SseEventStream;
use super::OpenCodeClient;

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
    pub(crate) failure_payload: Option<serde_json::Value>,
}

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
            failure_payload: None,
        }
    }

    pub async fn cleanup_server(&mut self) {
        if let Some(child) = self.server_process.take() {
            super::cleanup::stop_host_process(child).await;
        }

        let container_name = self.container_name.take();
        super::cleanup::remove_container(container_name.as_deref());
    }

    fn cleanup_server_sync(&mut self) {
        if let Some(child) = self.server_process.take() {
            super::cleanup::stop_host_process_sync(child);
        }

        let container_name = self.container_name.take();
        super::cleanup::remove_container(container_name.as_deref());
    }
}

impl Drop for OpenCodeRunningSession {
    fn drop(&mut self) {
        self.cleanup_server_sync();
    }
}
