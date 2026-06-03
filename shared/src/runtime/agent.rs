use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use uuid::Uuid;

use crate::client::ApiClient;
use crate::runtime::errors::HarnessError;
use crate::runtime::types::{AgentEvent, IsolatedEnvironment, SessionExport, SessionStatus};

pub trait RunningSession: Send {
    fn status(&self) -> SessionStatus;

    fn session_id(&self) -> Option<&str> {
        None
    }

    fn poll_event(&mut self) -> Pin<Box<dyn Future<Output = Option<AgentEvent>> + Send + '_>>;

    fn cancel(&mut self) -> Pin<Box<dyn Future<Output = Result<(), HarnessError>> + Send + '_>>;

    fn export(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<SessionExport, HarnessError>> + Send + '_>>;

    fn wait(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<SessionExport, HarnessError>> + Send + '_>>;

    fn continue_with(
        &mut self,
        prompt: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), HarnessError>> + Send + '_>>;

    fn set_event_reporter(&mut self, _client: Arc<ApiClient>, _token: String, _job_id: Uuid) {}
}

pub trait AgentRuntime: Send + Sync {
    fn execute(
        &self,
        prompt: &str,
        env: &IsolatedEnvironment,
        repo_url: &str,
    ) -> impl std::future::Future<Output = Result<Box<dyn RunningSession>, HarnessError>> + Send;
}
