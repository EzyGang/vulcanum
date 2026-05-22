#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum HarnessError {
    /// Docker or kata-runtime are missing, or the container image is not pulled.
    #[error("install error: {0}")]
    Install(String),

    /// The job exceeded its configured maximum duration.
    #[error("job timed out after {0}s")]
    Timeout(u64),

    /// OpenCode exited with a non-zero status or crashed.
    #[error("opencode crashed: {0}")]
    OpenCodeCrash(String),

    /// The output could not be parsed for PR URL or metrics.
    #[error("output parse error: {0}")]
    OutputParse(String),
}
