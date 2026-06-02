use std::future::Future;
use std::pin::Pin;

use crate::runtime::errors::HarnessError;
use crate::runtime::types::{AgentEvent, IsolatedEnvironment, SessionExport, SessionStatus};

pub trait RunningSession: Send {
    fn status(&self) -> SessionStatus;

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
}

pub trait AgentRuntime: Send + Sync {
    #[allow(clippy::too_many_arguments)]
    fn execute(
        &self,
        prompt: &str,
        env: &IsolatedEnvironment,
        repo_url: &str,
        agents_md: &str,
        opencode_config: &str,
    ) -> impl std::future::Future<Output = Result<Box<dyn RunningSession>, HarnessError>> + Send;
}
