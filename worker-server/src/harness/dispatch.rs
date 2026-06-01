use std::collections::HashMap;
use std::path::Path;

use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};
use vulcanum_shared::runtime::IsolationProvider;

use crate::harness::gvisor::GvisorIsolation;
use crate::harness::host::HostIsolation;
use crate::harness::kata::KataIsolation;

pub enum IsolationKind {
    Host(HostIsolation),
    Kata(KataIsolation),
    Gvisor(GvisorIsolation),
}

pub fn create_isolation_provider(harness_type: &str) -> IsolationKind {
    match harness_type {
        "kata" => {
            tracing::debug!("using Kata Containers isolation");
            IsolationKind::Kata(KataIsolation::new())
        }
        "gvisor" => {
            tracing::debug!("using gVisor isolation");
            IsolationKind::Gvisor(GvisorIsolation::new())
        }
        _ => {
            tracing::debug!("using host isolation");
            IsolationKind::Host(HostIsolation::new())
        }
    }
}

impl IsolationProvider for IsolationKind {
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
        match self {
            IsolationKind::Host(h) => {
                h.prepare(
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
            IsolationKind::Kata(k) => {
                k.prepare(
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
            IsolationKind::Gvisor(g) => {
                g.prepare(
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
        }
    }

    async fn cleanup(&self, env: &IsolatedEnvironment) {
        match self {
            IsolationKind::Host(h) => h.cleanup(env).await,
            IsolationKind::Kata(k) => k.cleanup(env).await,
            IsolationKind::Gvisor(g) => g.cleanup(env).await,
        }
    }
}
