use std::process::{Output, Stdio};
use std::time::Duration;

use tokio::process::Command;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

pub(super) async fn output(command: &mut Command) -> Option<Output> {
    command
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    match tokio::time::timeout(COMMAND_TIMEOUT, command.output()).await {
        Ok(Ok(output)) => Some(output),
        Ok(Err(error)) => {
            tracing::warn!(error = %error, "recovery command failed");
            None
        }
        Err(_) => {
            tracing::warn!("recovery command timed out");
            None
        }
    }
}
