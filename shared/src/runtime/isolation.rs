use std::collections::HashMap;
use std::future::Future;
use std::path::Path;

use crate::api_types::{AgentBackend, AgentConfigPayload, JobRepo, WorkRunType};
use crate::runtime::errors::HarnessError;
use crate::runtime::types::{IsolatedEnvironment, ResourceLimits};

pub trait IsolationProvider {
    #[allow(clippy::too_many_arguments)]
    fn prepare(
        &self,
        workdir: &Path,
        secrets: &HashMap<String, String>,
        env_vars: &HashMap<String, String>,
        limits: &ResourceLimits,
        work_type: WorkRunType,
        agents_md: &str,
        agent_backend: AgentBackend,
        agent_config: &AgentConfigPayload,
        repos: &[JobRepo],
    ) -> impl Future<Output = Result<IsolatedEnvironment, HarnessError>> + Send;

    fn cleanup(&self, env: &IsolatedEnvironment) -> impl Future<Output = ()> + Send;
}
