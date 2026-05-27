use std::collections::HashMap;

use uuid::Uuid;

use crate::api_error::ApiError;
use crate::client::{ApiClient, SubmitResultRequest};
use crate::harness::host::HostHarness;
use crate::harness::kata::KataHarness;
use crate::harness::{AgentHarness, HarnessKind};
use crate::state::WorkerState;

use super::{is_fatal_api_error, TickOutcome};

fn map_result(e: &anyhow::Error, op: &str, allow_404: bool, job_id: Uuid) -> TickOutcome {
    if is_fatal_api_error(e) {
        return TickOutcome::Fatal(format!("{op} failed: {:#}", e));
    }
    if allow_404 {
        if let Some(api_err) = e.downcast_ref::<ApiError>() {
            if api_err.status == 404 {
                tracing::info!(
                    work_run_id = %job_id,
                    "job was deleted or cancelled, skipping {op}"
                );
                return TickOutcome::Success;
            }
        }
    }
    TickOutcome::Transient(format!("{op} failed: {e:#}"))
}

pub(crate) async fn handle_job(
    client: &ApiClient,
    state: &WorkerState,
    job_id: Uuid,
) -> TickOutcome {
    tracing::info!(
        worker_id = %state.worker_id,
        work_run_id = %job_id,
        "job received",
    );

    let job = match client.get_job(job_id, &state.access_token).await {
        Ok(j) => j,
        Err(e) => return map_result(&e, "get_job", true, job_id),
    };

    if let Err(e) = client.ack_job(job_id, &state.access_token).await {
        return map_result(&e, "ack", true, job_id);
    }

    tracing::info!(
        worker_id = %state.worker_id,
        work_run_id = %job_id,
        external_task_ref = %job.external_task_ref,
        prompt_len = job.prompt_text.len(),
        "executing job",
    );

    let workdir = std::env::temp_dir().join(format!("vulcanum-work-{}", job_id));
    if let Err(e) = std::fs::create_dir_all(&workdir) {
        return TickOutcome::Fatal(format!("failed to create workdir: {e}"));
    }

    let harness = create_harness();
    let limits = crate::harness::ResourceLimits::default();
    let mut secrets = HashMap::new();
    secrets.insert("KANEO_INSTANCE".to_owned(), job.kaneo_instance);
    secrets.insert("KANEO_API_KEY".to_owned(), job.kaneo_api_key);
    secrets.insert("KANEO_PROJECT_ID".to_owned(), job.kaneo_project_id);
    secrets.insert("KANEO_WORKSPACE_ID".to_owned(), job.kaneo_workspace_id);
    secrets.insert("KANEO_TASK_ID".to_owned(), job.external_task_ref.clone());

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
        Err(_e) => {
            tracing::error!(
                worker_id = %state.worker_id,
                work_run_id = %job_id,
                external_task_ref = %job.external_task_ref,
                "job execution failed",
            );
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
                tracing::error!(
                    worker_id = %state.worker_id,
                    work_run_id = %job_id,
                    "submit_result failed for job",
                );
                return map_result(&e, "submit_result", false, job_id);
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
        return map_result(&e, "submit_result", false, job_id);
    }

    tracing::info!(
        worker_id = %state.worker_id,
        work_run_id = %job_id,
        external_task_ref = %job.external_task_ref,
        tokens_used = harness_result.tokens_used,
        duration_ms = harness_result.duration_ms,
        exit_code = harness_result.exit_code,
        "job completed",
    );

    let _ = std::fs::remove_dir_all(&workdir);

    TickOutcome::Success
}

pub(crate) fn create_harness() -> HarnessKind {
    let harness_type = std::env::var("VULCANUM_HARNESS").unwrap_or_else(|_| "host".to_owned());

    match harness_type.as_str() {
        "kata" => {
            tracing::debug!("using Kata Containers harness");
            HarnessKind::Kata(KataHarness::new())
        }
        _ => {
            tracing::debug!("using host harness");
            HarnessKind::Host(HostHarness::new())
        }
    }
}
