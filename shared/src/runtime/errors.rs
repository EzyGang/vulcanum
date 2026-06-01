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
}
