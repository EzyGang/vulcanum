use std::collections::HashMap;
use std::path::Path;

use vulcanum_shared::api_types::{JobRepo, WorkRunType};
use vulcanum_shared::config::WorkerConfig;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};

use crate::isolation::providers::docker::DockerIsolation;
use crate::isolation::providers::host::HostIsolation;
use crate::isolation::providers::kata::KataIsolation;

pub enum IsolationKind {
    Host(HostIsolation),
    Kata(KataIsolation),
    Docker(DockerIsolation),
}

pub fn create_isolation_provider(config: &WorkerConfig) -> IsolationKind {
    let harness_type = config.harness.as_str();
    match harness_type {
        "kata" => {
            tracing::debug!("using Kata Containers isolation");
            IsolationKind::Kata(KataIsolation::new(config.image.clone()))
        }
        "docker" => {
            tracing::debug!("using Docker isolation");
            IsolationKind::Docker(DockerIsolation::new(None, config.image.clone()))
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
        work_type: WorkRunType,
        agents_md: &str,
        generated_opencode_config: &str,
        repos: &[JobRepo],
    ) -> Result<IsolatedEnvironment, HarnessError> {
        match self {
            IsolationKind::Host(h) => {
                h.prepare(
                    workdir,
                    secrets,
                    env_vars,
                    limits,
                    work_type,
                    agents_md,
                    generated_opencode_config,
                    repos,
                )
                .await
            }
            IsolationKind::Kata(k) => {
                k.prepare(
                    workdir,
                    secrets,
                    env_vars,
                    limits,
                    work_type,
                    agents_md,
                    generated_opencode_config,
                    repos,
                )
                .await
            }
            IsolationKind::Docker(d) => {
                d.prepare(
                    workdir,
                    secrets,
                    env_vars,
                    limits,
                    work_type,
                    agents_md,
                    generated_opencode_config,
                    repos,
                )
                .await
            }
        }
    }

    async fn cleanup(&self, env: &IsolatedEnvironment) {
        match self {
            IsolationKind::Host(h) => h.cleanup(env).await,
            IsolationKind::Kata(k) => k.cleanup(env).await,
            IsolationKind::Docker(d) => d.cleanup(env).await,
        }
    }
}
