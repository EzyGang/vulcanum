pub mod errors;
pub mod host;
pub mod kata;
mod parse;
pub(crate) mod runner;

#[cfg(test)]
mod errors_tests;

#[cfg(test)]
mod host_tests;

#[cfg(test)]
mod kata_tests;

#[cfg(test)]
mod parse_tests;

use std::collections::HashMap;
use std::path::Path;

use crate::harness::errors::HarnessError;
use crate::harness::host::HostHarness;
use crate::harness::kata::KataHarness;

/// The result of a completed agent job.
#[derive(Debug, Clone, PartialEq)]
pub struct HarnessResult {
    pub exit_code: i32,
    pub tokens_used: u64,
    pub pr_url: Option<String>,
    pub duration_ms: u64,
}

/// Resource limits applied to a single job execution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResourceLimits {
    pub max_duration_secs: u64,
    pub vcpu_count: u64,
    pub memory_mib: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_duration_secs: 1_800,
            vcpu_count: 2,
            memory_mib: 1_024,
        }
    }
}

/// Enum dispatch over harness implementations.
///
/// Allows the daemon to select between host and Kata harnesses at runtime
/// without requiring trait objects or the async_trait crate.
pub enum HarnessKind {
    Host(HostHarness),
    Kata(KataHarness),
}

impl AgentHarness for HarnessKind {
    async fn spawn(
        &self,
        prompt: &str,
        workdir: &Path,
        secrets: &HashMap<String, String>,
        limits: &ResourceLimits,
        repo_url: &str,
        agents_md: &str,
    ) -> Result<HarnessResult, HarnessError> {
        match self {
            Self::Host(h) => {
                h.spawn(prompt, workdir, secrets, limits, repo_url, agents_md)
                    .await
            }
            Self::Kata(k) => {
                k.spawn(prompt, workdir, secrets, limits, repo_url, agents_md)
                    .await
            }
        }
    }
}

/// Abstract contract for spawning an agent job executor.
///
/// Implementations may run on the host (HostHarness) or inside a Kata
/// Containers VM (KataHarness) via Docker with --runtime=kata-runtimes.
pub trait AgentHarness {
    /// Spawn the job, returning the result once the agent exits.
    fn spawn(
        &self,
        prompt: &str,
        workdir: &Path,
        secrets: &HashMap<String, String>,
        limits: &ResourceLimits,
        repo_url: &str,
        agents_md: &str,
    ) -> impl std::future::Future<Output = Result<HarnessResult, HarnessError>> + Send;
}
