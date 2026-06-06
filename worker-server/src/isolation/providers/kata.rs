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
        agents_md: &str,
        opencode_config: &str,
        repo_url: &str,
    ) -> Result<IsolatedEnvironment, vulcanum_shared::runtime::errors::HarnessError> {
        self.inner
            .prepare(
                workdir,
                secrets,
                env_vars,
                limits,
                agents_md,
                opencode_config,
                repo_url,
            )
            .await
    }

    async fn cleanup(&self, env: &IsolatedEnvironment) {
        self.inner.cleanup(env).await;
    }
}
