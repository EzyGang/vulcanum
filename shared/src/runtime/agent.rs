use crate::runtime::errors::HarnessError;
use crate::runtime::types::{IsolatedEnvironment, SessionExport};

pub trait AgentRuntime: Send + Sync {
    fn execute(
        &self,
        prompt: &str,
        env: &IsolatedEnvironment,
        repo_url: &str,
        agents_md: &str,
        opencode_config: &str,
    ) -> impl std::future::Future<Output = Result<SessionExport, HarnessError>> + Send;
}
