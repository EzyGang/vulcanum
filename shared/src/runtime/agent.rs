use std::future::Future;
use std::pin::Pin;

use crate::runtime::errors::HarnessError;
use crate::runtime::types::{AgentEvent, IsolatedEnvironment, SessionExport, SessionStatus};

pub trait RunningSession: Send {
    fn status(&self) -> SessionStatus;

    fn session_id(&self) -> Option<&str> {
        None
    }

    fn agent_session_path(&self) -> Option<&str> {
        None
    }

    fn agent_pid(&self) -> Option<u32> {
        None
    }

    fn agent_base_url(&self) -> Option<&str> {
        None
    }

    fn poll_event(&mut self) -> Pin<Box<dyn Future<Output = Option<AgentEvent>> + Send + '_>>;

    fn cancel(&mut self) -> Pin<Box<dyn Future<Output = Result<(), HarnessError>> + Send + '_>>;

    fn cleanup(&mut self) -> Pin<Box<dyn Future<Output = Result<(), HarnessError>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }

    fn export(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<SessionExport, HarnessError>> + Send + '_>>;

    fn export_messages(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<serde_json::Value>, HarnessError>> + Send + '_>>
    {
        Box::pin(async { Ok(None) })
    }

    fn wait(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<SessionExport, HarnessError>> + Send + '_>>;

    fn continue_with(
        &mut self,
        prompt: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), HarnessError>> + Send + '_>>;

    fn host_server_info(&self) -> Option<(u32, u16)> {
        None
    }
}

pub trait AgentRuntime: Send + Sync {
    fn execute(
        &self,
        prompt: &str,
        env: &IsolatedEnvironment,
    ) -> impl Future<Output = Result<Box<dyn RunningSession>, HarnessError>> + Send;
}
