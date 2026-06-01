#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    #[error("install error: {0}")]
    Install(String),

    #[error("job timed out after {0}s")]
    Timeout(u64),

    #[error("agent crashed: {0}")]
    Crash(String),

    #[error("output parse error: {0}")]
    OutputParse(String),

    #[error("server launch failed: {0}")]
    ServerLaunch(String),

    #[error("server unhealthy: {0}")]
    ServerUnhealthy(String),

    #[error("stall detected: no event for {0}s")]
    StallDetected(u64),

    #[error("cancel failed: {0}")]
    CancelFailed(String),

    #[error("http error: {0}")]
    Http(String),
}
