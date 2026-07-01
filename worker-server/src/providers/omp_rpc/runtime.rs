use std::path::Path;

use tokio::sync::mpsc;
use vulcanum_shared::runtime::agent::{AgentRuntime, RunningSession};
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::IsolatedEnvironment;

use crate::providers::omp_rpc::process::{
    launch_omp, read_stderr_tail, read_stdout_frames, ProcessOutputBuffer,
};
use crate::providers::omp_rpc::session::OmpRpcRunningSession;

pub struct OmpRpcRuntime;

impl OmpRpcRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub async fn resume(
        &self,
        prompt: &str,
        env: &IsolatedEnvironment,
        session_path: &Path,
    ) -> Result<Box<dyn RunningSession>, HarnessError> {
        self.start(prompt, env, Some(session_path)).await
    }

    async fn start(
        &self,
        prompt: &str,
        env: &IsolatedEnvironment,
        resume_path: Option<&Path>,
    ) -> Result<Box<dyn RunningSession>, HarnessError> {
        let mut child = launch_omp(env, resume_path).await?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| HarnessError::ServerLaunch("omp stdout was not piped".to_owned()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| HarnessError::ServerLaunch("omp stdin was not piped".to_owned()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| HarnessError::ServerLaunch("omp stderr was not piped".to_owned()))?;
        let (tx, rx) = mpsc::channel(256);
        let stderr_buffer = ProcessOutputBuffer::default();
        tokio::spawn(read_stdout_frames(stdout, tx));
        tokio::spawn(read_stderr_tail(stderr, stderr_buffer.clone()));

        let mut running = OmpRpcRunningSession::new(
            child,
            stdin,
            rx,
            stderr_buffer,
            env.limits.max_duration_secs,
        );
        running.wait_ready().await?;
        running.refresh_state(env).await?;
        running
            .send_command(serde_json::json!({
                "id": "prompt-1",
                "type": "prompt",
                "message": prompt,
            }))
            .await?;
        running.wait_for_response("prompt-1", "prompt").await?;

        Ok(Box::new(running))
    }
}

impl AgentRuntime for OmpRpcRuntime {
    async fn execute(
        &self,
        prompt: &str,
        env: &IsolatedEnvironment,
    ) -> Result<Box<dyn RunningSession>, HarnessError> {
        self.start(prompt, env, None).await
    }
}
