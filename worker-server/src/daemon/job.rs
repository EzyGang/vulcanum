use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use vulcanum_shared::api_error::{is_fatal_api_error, ApiError};
use vulcanum_shared::api_types::SubmitResultRequest;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::worker_state::WorkerState;

use crate::harness::gvisor::GvisorHarness;
use crate::harness::host::HostHarness;
use crate::harness::kata::KataHarness;
use crate::harness::{AgentHarness, HarnessKind};
use crate::state::journal::{Journal, JournalStatus};

pub(crate) async fn handle_job(
    client: Arc<ApiClient>,
    worker_state: Arc<RwLock<WorkerState>>,
    journal: Arc<Journal>,
    job_id: Uuid,
    harness_type: &str,
) -> Result<(), String> {
    let worker_id = worker_state.read().await.worker_id;

    tracing::info!(
        worker_id = %worker_id,
        work_run_id = %job_id,
        "job received",
    );

    let access_token = worker_state.read().await.access_token.clone();

    let job = match client.get_job(job_id, &access_token).await {
        Ok(j) => j,
        Err(e) => {
            if is_fatal_api_error(&e) {
                return Err(format!("get_job failed permanently: {e:#} — run `vulcanum worker setup --instance <instance> --code <code>` to reconnect"));
            }
            if let Some(api_err) = e.downcast_ref::<ApiError>() {
                if api_err.status == 404 {
                    tracing::info!(work_run_id = %job_id, "job was deleted or cancelled, skipping");
                    return Ok(());
                }
            }
            tracing::warn!(work_run_id = %job_id, error = %e, "get_job failed transiently, skipping");
            return Ok(());
        }
    };

    if let Err(e) = client.ack_job(job_id, &access_token).await {
        if is_fatal_api_error(&e) {
            return Err(format!("ack failed permanently: {e:#} — run `vulcanum worker setup --instance <instance> --code <code>` to reconnect"));
        }
        if let Some(api_err) = e.downcast_ref::<ApiError>() {
            if api_err.status == 404 {
                tracing::info!(work_run_id = %job_id, "job was deleted or cancelled, skipping ack");
                return Ok(());
            }
        }
        tracing::warn!(work_run_id = %job_id, error = %e, "ack failed transiently, skipping");
        return Ok(());
    }

    tracing::info!(
        worker_id = %worker_id,
        work_run_id = %job_id,
        external_task_ref = %job.external_task_ref,
        prompt_len = job.prompt_text.len(),
        "executing job",
    );

    let workdir = std::env::temp_dir().join(format!("vulcanum-work-{}", job_id));
    if let Err(e) = std::fs::create_dir_all(&workdir) {
        tracing::error!(work_run_id = %job_id, error = %e, "failed to create workdir");
        return Ok(());
    }

    let workdir_str = workdir.to_string_lossy().to_string();

    let container_name = match harness_type {
        "kata" | "gvisor" => {
            let file_name = workdir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("job");
            Some(format!("vulcanum-{file_name}"))
        }
        _ => None,
    };

    let started_at = Utc::now();
    let _ = journal.insert_job(
        job_id,
        &workdir_str,
        container_name.as_deref(),
        harness_type,
        started_at,
    );

    let harness = create_harness(harness_type);
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
            &job.opencode_config,
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(
                worker_id = %worker_id,
                work_run_id = %job_id,
                external_task_ref = %job.external_task_ref,
                error = %e,
                "job execution failed",
            );
            let _ = journal.update_result(job_id, 1, 0, None, 0, JournalStatus::Failed);
            let result = SubmitResultRequest {
                pr_url: String::new(),
                exit_code: 1,
                tokens_used: 0,
                duration_ms: 0,
            };
            let access_token = worker_state.read().await.access_token.clone();
            if let Err(e) = client.submit_result(job_id, &result, &access_token).await {
                tracing::error!(
                    worker_id = %worker_id,
                    work_run_id = %job_id,
                    error = %e,
                    "submit_result failed for job",
                );
            }
            let _ = journal.mark_submitted(job_id);
            return Ok(());
        }
    };

    let pr_url = harness_result.pr_url.clone();
    let result = SubmitResultRequest {
        pr_url: pr_url.unwrap_or_default(),
        exit_code: harness_result.exit_code,
        tokens_used: harness_result.tokens_used as i64,
        duration_ms: harness_result.duration_ms as i64,
    };

    let journal_status = if harness_result.exit_code == 0 {
        JournalStatus::Completed
    } else {
        JournalStatus::Failed
    };

    let _ = journal.update_result(
        job_id,
        harness_result.exit_code,
        harness_result.tokens_used as i64,
        harness_result.pr_url.as_deref(),
        harness_result.duration_ms as i64,
        journal_status,
    );

    let access_token = worker_state.read().await.access_token.clone();
    if let Err(e) = client.submit_result(job_id, &result, &access_token).await {
        if is_fatal_api_error(&e) {
            tracing::error!(
                worker_id = %worker_id,
                work_run_id = %job_id,
                error = %e,
                "submit_result failed permanently"
            );
            return Err(format!("submit_result failed permanently: {e:#}"));
        }
        tracing::warn!(
            worker_id = %worker_id,
            work_run_id = %job_id,
            error = %e,
            "submit_result failed transiently, result not persisted"
        );
        return Ok(());
    }

    let _ = journal.mark_submitted(job_id);

    tracing::info!(
        worker_id = %worker_id,
        work_run_id = %job_id,
        external_task_ref = %job.external_task_ref,
        tokens_used = harness_result.tokens_used,
        duration_ms = harness_result.duration_ms,
        exit_code = harness_result.exit_code,
        "job completed",
    );

    let _ = std::fs::remove_dir_all(&workdir);

    Ok(())
}

pub(crate) fn create_harness(harness_type: &str) -> HarnessKind {
    match harness_type {
        "kata" => {
            tracing::debug!("using Kata Containers harness");
            HarnessKind::Kata(KataHarness::new())
        }
        "gvisor" => {
            tracing::debug!("using gVisor harness");
            HarnessKind::Gvisor(GvisorHarness::new())
        }
        _ => {
            tracing::debug!("using host harness");
            HarnessKind::Host(HostHarness::new())
        }
    }
}
