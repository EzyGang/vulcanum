use std::collections::HashMap;
use std::path::Path;

use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};
use vulcanum_shared::runtime::IsolationProvider;

use crate::harness::container::DockerIsolation;

pub struct KataIsolation {
    pub(crate) inner: DockerIsolation,
}

impl KataIsolation {
    pub fn new() -> Self {
        Self {
            inner: DockerIsolation::new("kata-runtime"),
        }
    }

    #[allow(dead_code)]
    pub fn with_image(image: String) -> Self {
        Self {
            inner: DockerIsolation::with_image(image, "kata-runtime"),
        }
    }
}

impl Default for KataIsolation {
    fn default() -> Self {
        Self::new()
    }
}

impl IsolationProvider for KataIsolation {
    async fn prepare(
        &self,
        workdir: &Path,
        secrets: &HashMap<String, String>,
        env_vars: &HashMap<String, String>,
        limits: &ResourceLimits,
        agents_md: &str,
        opencode_config: &str,
        repo_url: &str,
    ) -> Result<IsolatedEnvironment, HarnessError> {
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
