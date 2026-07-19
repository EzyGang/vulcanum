use vulcanum_shared::api::wire::AgentBackend;
use vulcanum_shared::runtime::agent::{AgentRuntime, RunningSession};
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::IsolatedEnvironment;

use crate::providers::omp_rpc::runtime::OmpRpcRuntime;
use crate::providers::opencode::runtime::OpenCodeServeRuntime;

pub enum AgentRuntimeKind {
    OpenCode(OpenCodeServeRuntime),
    OmpRpc(OmpRpcRuntime),
}

impl AgentRuntimeKind {
    #[must_use]
    pub fn new(backend: AgentBackend) -> Self {
        match backend {
            AgentBackend::OpenCode => Self::OpenCode(OpenCodeServeRuntime::new()),
            AgentBackend::OmpRpc => Self::OmpRpc(OmpRpcRuntime::new()),
        }
    }

    pub async fn execute(
        &self,
        prompt: &str,
        env: &IsolatedEnvironment,
    ) -> Result<Box<dyn RunningSession>, HarnessError> {
        match self {
            Self::OpenCode(runtime) => runtime.execute(prompt, env).await,
            Self::OmpRpc(runtime) => runtime.execute(prompt, env).await,
        }
    }
}
