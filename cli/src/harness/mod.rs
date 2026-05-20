pub mod errors;
pub mod host;
pub mod validate;

#[cfg(test)]
mod errors_tests;

#[cfg(test)]
mod host_tests;

use std::collections::HashMap;
use std::path::Path;

use crate::harness::errors::HarnessError;

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

/// Abstract contract for spawning an agent job executor.
///
/// Implementations may run on the host (HostHarness) or inside a microVM
/// (FirecrackerHarness, deferred to a follow-up ticket).
pub trait AgentHarness {
    /// Spawn the job, returning the result once the agent exits.
    fn spawn(
        &self,
        prompt: &str,
        workdir: &Path,
        secrets: &HashMap<String, String>,
        limits: &ResourceLimits,
    ) -> impl std::future::Future<Output = Result<HarnessResult, HarnessError>> + Send;
}
