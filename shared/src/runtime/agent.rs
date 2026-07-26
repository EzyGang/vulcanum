use std::future::Future;
use std::pin::Pin;

use crate::runtime::errors::HarnessError;
use crate::runtime::types::{AgentEvent, IsolatedEnvironment, SessionExport, SessionStatus};

#[must_use]
pub(super) fn value_contains_text(value: &serde_json::Value, expected: &str) -> bool {
    match value {
        serde_json::Value::String(text) => text == expected,
        serde_json::Value::Array(values) => values
            .iter()
            .any(|value| value_contains_text(value, expected)),
        serde_json::Value::Object(values) => values
            .values()
            .any(|value| value_contains_text(value, expected)),
        _ => false,
    }
}

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

    fn prompt_was_dispatched(
        &mut self,
        prompt: &str,
    ) -> Pin<Box<dyn Future<Output = Result<bool, HarnessError>> + Send + '_>> {
        let prompt = prompt.to_owned();
        Box::pin(async move {
            let messages = self.export_messages().await?;
            Ok(messages
                .as_ref()
                .is_some_and(|messages| value_contains_text(messages, &prompt)))
        })
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
