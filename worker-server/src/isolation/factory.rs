use std::collections::HashMap;
use std::path::Path;

use vulcanum_shared::api::wire::{AgentBackend, AgentConfigPayload, JobRepo, WorkRunType};
use vulcanum_shared::config::{IsolationBackend, WorkerConfig};
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

pub fn create_isolation_provider(config: &WorkerConfig) -> Result<IsolationKind, HarnessError> {
    match config.isolation_backend() {
        Ok(IsolationBackend::Host) => {
            tracing::debug!("using host isolation");
            Ok(IsolationKind::Host(HostIsolation::new()))
        }
        Ok(IsolationBackend::Kata) => {
            tracing::debug!("using Kata Containers isolation");
            Ok(IsolationKind::Kata(KataIsolation::new(
                config.image.clone(),
            )))
        }
        Ok(IsolationBackend::Docker) => {
            tracing::debug!("using Docker isolation");
            Ok(IsolationKind::Docker(DockerIsolation::new(
                None,
                config.image.clone(),
            )))
        }
        Err(err) => Err(HarnessError::Crash(err.to_string())),
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
        agent_backend: AgentBackend,
        agent_config: &AgentConfigPayload,
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
                    agent_backend,
                    agent_config,
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
                    agent_backend,
                    agent_config,
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
                    agent_backend,
                    agent_config,
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

#[cfg(test)]
mod tests {
    use vulcanum_shared::config::WorkerConfig;
    use vulcanum_shared::runtime::errors::HarnessError;

    use crate::isolation::factory::{create_isolation_provider, IsolationKind};

    #[test]
    fn create_isolation_provider_accepts_explicit_host() {
        let config = WorkerConfig {
            harness: "host".to_owned(),
            ..WorkerConfig::default()
        };

        let provider = create_isolation_provider(&config).expect("host harness should be valid");

        match provider {
            IsolationKind::Host(_) => (),
            IsolationKind::Kata(_) | IsolationKind::Docker(_) => {
                panic!("explicit host harness should create host isolation")
            }
        }
    }

    #[test]
    fn create_isolation_provider_rejects_unknown_harness() {
        let config = WorkerConfig {
            harness: "firecracker".to_owned(),
            ..WorkerConfig::default()
        };

        let err = match create_isolation_provider(&config) {
            Ok(_) => panic!("unknown harness should fail"),
            Err(err) => err,
        };

        match err {
            HarnessError::Crash(message) => {
                assert!(message.contains("unknown isolation backend \"firecracker\""));
                assert!(message.contains("host, docker, kata"));
            }
            HarnessError::Install(_)
            | HarnessError::Timeout(_)
            | HarnessError::OutputParse(_)
            | HarnessError::ServerLaunch(_)
            | HarnessError::ServerUnhealthy(_)
            | HarnessError::StallDetected(_)
            | HarnessError::CancelFailed(_)
            | HarnessError::Http(_) => {
                panic!("unknown harness should fail as a crash configuration error")
            }
        }
    }
}
