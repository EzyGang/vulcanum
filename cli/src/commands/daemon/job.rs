use std::collections::HashMap;

use uuid::Uuid;

use crate::client::{ApiClient, SubmitResultRequest};
use crate::harness::host::HostHarness;
use crate::harness::kata::KataHarness;
use crate::harness::{AgentHarness, HarnessKind};
use crate::state::WorkerState;

use super::{is_fatal_api_error, TickOutcome};

pub(crate) async fn handle_job(
    client: &ApiClient,
    state: &WorkerState,
    job_id: Uuid,
) -> TickOutcome {
    tracing::info!("job received: {}", job_id);

    let job = match client.get_job(job_id, &state.access_token).await {
        Ok(j) => j,
        Err(e) => {
            if is_fatal_api_error(&e) {
                return TickOutcome::Fatal(format!("get_job failed: {:#}", e));
            }
            return TickOutcome::Transient;
        }
    };

    if let Err(e) = client.ack_job(job_id, &state.access_token).await {
        if is_fatal_api_error(&e) {
            return TickOutcome::Fatal(format!("ack failed: {:#}", e));
        }
        return TickOutcome::Transient;
    }

    tracing::info!(
        "executing job {} (task: {}, prompt length: {})",
        job_id,
        job.external_task_ref,
        job.prompt_text.len()
    );

    let workdir = std::env::temp_dir().join(format!("vulcanum-work-{}", job_id));
    if let Err(e) = std::fs::create_dir_all(&workdir) {
        return TickOutcome::Fatal(format!("failed to create workdir: {e}"));
    }

    let harness = create_harness();
    let limits = crate::harness::ResourceLimits::default();
    let secrets = HashMap::new();

    let harness_result = match harness
        .spawn(
            &job.prompt_text,
            &workdir,
            &secrets,
            &limits,
            &job.repo_url,
            &job.agents_md,
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("job {} execution failed: {}", job_id, e);
            let result = SubmitResultRequest {
                pr_url: String::new(),
                exit_code: 1,
                tokens_used: 0,
                duration_ms: 0,
            };
            if let Err(e) = client
                .submit_result(job_id, &result, &state.access_token)
                .await
            {
                if is_fatal_api_error(&e) {
                    return TickOutcome::Fatal(format!("submit_result failed: {:#}", e));
                }
                return TickOutcome::Transient;
            }
            return TickOutcome::Success;
        }
    };

    let result = SubmitResultRequest {
        pr_url: harness_result.pr_url.unwrap_or_default(),
        exit_code: harness_result.exit_code,
        tokens_used: harness_result.tokens_used as i64,
        duration_ms: harness_result.duration_ms as i64,
    };

    if let Err(e) = client
        .submit_result(job_id, &result, &state.access_token)
        .await
    {
        if is_fatal_api_error(&e) {
            return TickOutcome::Fatal(format!("submit_result failed: {:#}", e));
        }
        return TickOutcome::Transient;
    }

    tracing::info!(
        "job {} completed in {}ms (exit: {})",
        job_id,
        harness_result.duration_ms,
        harness_result.exit_code
    );

    let _ = std::fs::remove_dir_all(&workdir);

    TickOutcome::Success
}

pub(crate) fn create_harness() -> HarnessKind {
    let harness_type = std::env::var("VULCANUM_HARNESS").unwrap_or_else(|_| "host".to_owned());

    match harness_type.as_str() {
        "kata" => {
            tracing::info!("using Kata Containers harness");
            HarnessKind::Kata(KataHarness::new())
        }
        _ => {
            tracing::info!("using host harness");
            HarnessKind::Host(HostHarness::new())
        }
    }
}
