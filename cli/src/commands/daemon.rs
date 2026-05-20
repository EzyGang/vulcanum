use std::time::Duration;

use tokio::signal;
use tokio::time::sleep;

use crate::client::{ApiClient, SubmitResultRequest};
use crate::state::{load_state, WorkerState};
use crate::token::ensure_valid_token;

const POLL_INTERVAL_SECS: u64 = 5;
const INITIAL_BACKOFF_MS: u64 = 1_000;
const MAX_BACKOFF_MS: u64 = 60_000;
const BACKOFF_MULTIPLIER: u64 = 2;

pub async fn run() -> anyhow::Result<()> {
    let mut state = load_state()?.ok_or_else(|| {
        anyhow::anyhow!(
            "no worker state found — run `vulcanum worker connect <instance> --code <code>` first"
        )
    })?;

    let client = ApiClient::new(state.instance_url.clone());

    tracing::info!("daemon started, worker_id: {}", state.worker_id);

    let mut backoff_ms = INITIAL_BACKOFF_MS;

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                tracing::info!("received SIGINT, shutting down");
                return Ok(());
            }
            result = tick(&client, &mut state, &mut backoff_ms) => {
                if let Err(e) = result {
                    tracing::error!("tick error: {:#}", e);
                    sleep(Duration::from_millis(backoff_ms)).await;
                    backoff_ms = (backoff_ms * BACKOFF_MULTIPLIER).min(MAX_BACKOFF_MS);
                }
            }
        }
    }
}

async fn tick(
    client: &ApiClient,
    state: &mut WorkerState,
    backoff_ms: &mut u64,
) -> anyhow::Result<()> {
    ensure_valid_token(client, state).await?;

    match client.poll(state.worker_id, &state.access_token).await? {
        None => {
            sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
            Ok(())
        }
        Some(job_id) => {
            tracing::info!("job received: {}", job_id);
            *backoff_ms = INITIAL_BACKOFF_MS;

            let job = client.get_job(job_id, &state.access_token).await?;

            client
                .ack_job(job_id, state.worker_id, &state.access_token)
                .await?;

            tracing::info!(
                "executing job {} (task: {}, prompt length: {})",
                job_id,
                job.external_task_ref,
                job.prompt_text.len()
            );

            let start = std::time::Instant::now();
            let _ = job.prompt_text;

            // Placeholder: actual execution via OpenCode harness (VLC-10)
            let exit_code: i32 = 0;
            let elapsed = start.elapsed();

            let result = SubmitResultRequest {
                pr_url: String::new(),
                exit_code,
                tokens_used: 0,
                duration_ms: elapsed.as_millis() as i32,
            };

            client
                .submit_result(job_id, &result, &state.access_token)
                .await?;

            tracing::info!("job {} completed in {}ms", job_id, elapsed.as_millis());

            Ok(())
        }
    }
}
