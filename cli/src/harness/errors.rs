#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum HarnessError {
    /// Firecracker or jailer binaries are missing, or rootfs image not found.
    #[error("install error: {0}")]
    Install(String),

    /// The VM failed to boot, or its config was rejected by Firecracker.
    #[error("vm boot error: {0}")]
    VmBoot(String),

    /// The job exceeded its configured maximum duration.
    #[error("job timed out after {0}s")]
    VmTimeout(u64),

    /// OpenCode exited with a non-zero status or crashed.
    #[error("opencode crashed: {0}")]
    OpenCodeCrash(String),

    /// The output could not be parsed for PR URL or metrics.
    #[error("output parse error: {0}")]
    OutputParse(String),
}
