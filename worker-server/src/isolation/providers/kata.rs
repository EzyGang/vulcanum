use vulcanum_shared::api::wire::{AgentBackend, AgentConfigPayload, JobRepo, WorkRunType};
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};

use crate::isolation::providers::docker::DockerIsolation;

pub struct KataIsolation {
    pub(crate) inner: DockerIsolation,
}

impl KataIsolation {
    pub fn new(image: String) -> Self {
        Self {
            inner: DockerIsolation::new(Some("kata-runtime"), image),
        }
    }

    #[allow(dead_code)]
    pub fn with_image(image: String) -> Self {
        Self {
            inner: DockerIsolation::with_image(image, Some("kata-runtime")),
        }
    }
}

impl IsolationProvider for KataIsolation {
    async fn prepare(
        &self,
        workdir: &std::path::Path,
        secrets: &std::collections::HashMap<String, String>,
        env_vars: &std::collections::HashMap<String, String>,
        limits: &ResourceLimits,
        work_type: WorkRunType,
        agents_md: &str,
        agent_backend: AgentBackend,
        agent_config: &AgentConfigPayload,
        repos: &[JobRepo],
    ) -> Result<IsolatedEnvironment, vulcanum_shared::runtime::errors::HarnessError> {
        self.inner
            .prepare(
                workdir,
                secrets,
                env_vars,
                limits,
                work_type,
                agents_md,
                agent_backend,
                agent_config,
                repos,
            )
            .await
    }

    async fn cleanup(&self, env: &IsolatedEnvironment) {
        self.inner.cleanup(env).await;
    }
}
