use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use vulcanum_shared::api_error::{is_fatal_api_error, ApiError};
use vulcanum_shared::api_types::SubmitResultRequest;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::runtime::types::ResourceLimits;
use vulcanum_shared::runtime::IsolationProvider;
use vulcanum_shared::worker_state::WorkerState;

use crate::harness::dispatch::create_isolation_provider;
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
            let file_name = workdir.file_name().and_then(|n| n.to_str());
            Some(format!("vulcanum-{}", file_name.unwrap_or("job")))
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

    let provider = create_isolation_provider(harness_type);
    let limits = ResourceLimits::default();
    let mut secrets = HashMap::new();
    secrets.insert("KANEO_INSTANCE".to_owned(), job.kaneo_instance);
    secrets.insert("KANEO_API_KEY".to_owned(), job.kaneo_api_key);
    secrets.insert("KANEO_PROJECT_ID".to_owned(), job.kaneo_project_id);
    secrets.insert("KANEO_WORKSPACE_ID".to_owned(), job.kaneo_workspace_id);
    secrets.insert("KANEO_TASK_ID".to_owned(), job.external_task_ref.clone());
    let env_vars = HashMap::new();

    let _isolated_env = match provider
        .prepare(
            &workdir,
            &secrets,
            &env_vars,
            &limits,
            &job.agents_md,
            &job.opencode_config,
            &job.repo_url,
        )
        .await
    {
        Ok(env) => env,
        Err(e) => {
            tracing::error!(
                worker_id = %worker_id,
                work_run_id = %job_id,
                external_task_ref = %job.external_task_ref,
                error = %e,
                "isolation prepare failed",
            );
            let _ = journal.update_result(job_id, 1, 0, None, 0, JournalStatus::Failed);
            let result = SubmitResultRequest {
                pr_url: String::new(),
                exit_code: 1,
                tokens_used: 0,
                duration_ms: 0,
                input_tokens: 0,
                output_tokens: 0,
                cache_read_tokens: 0,
                cache_write_tokens: 0,
                model_used: None,
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

    todo!("AgentRuntime not yet implemented — see VLC-40");
}
