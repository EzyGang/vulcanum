use std::collections::HashMap;
use std::path::Path;

use crate::api_types::JobRepo;
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
        agents_md: &str,
        generated_opencode_config: &str,
        repos: &[JobRepo],
    ) -> impl std::future::Future<Output = Result<IsolatedEnvironment, HarnessError>> + Send;

    fn cleanup(&self, env: &IsolatedEnvironment) -> impl std::future::Future<Output = ()> + Send;
}
