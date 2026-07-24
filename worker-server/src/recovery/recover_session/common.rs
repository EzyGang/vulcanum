use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use vulcanum_shared::api::error::ApiError;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::constants::DEFAULT_IMAGE;
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};
use vulcanum_shared::state::worker::WorkerState;

use crate::daemon::auth::with_retry_on_401;
use crate::daemon::job::execution::submit::{submit_result_request, SubmitResultParams};
use crate::isolation::providers::docker::DockerIsolation;
use crate::isolation::providers::host::HostIsolation;
use crate::isolation::providers::kata::KataIsolation;
use crate::recovery::cleanup::kill_host_process_group;
use crate::state::journal::{Journal, JournalEntry, JournalResultUpdate, JournalStatus};
use crate::storage::messages::MessageStore;

pub(super) fn recovery_continuation_prompt(turn: i32, max_turns: i32) -> String {
    let next_turn = turn + 1;
    let final_turn_instruction = match next_turn >= max_turns {
        true => " This is the final allowed turn; before stopping, call the finish_run tool.",
        false => "",
    };

    format!(
        "[Continuation turn {next_turn}/{max_turns}]\n\
         The previous turn completed. The task remains active. \
         Continue from the current workspace state. Do not restart. \
         The workspace may contain multiple sibling repositories; run commands from the relevant repo directory. \
         Focus on remaining work. When done, call the finish_run tool.{final_turn_instruction}"
    )
}

pub(super) fn save_recovered_messages(
    job_id: uuid::Uuid,
    session_id: &str,
    messages: &serde_json::Value,
) {
    match MessageStore::new() {
        Ok(store) => {
            if let Err(e) = store.save(job_id, session_id, messages) {
                tracing::warn!(
                    work_run_id = %job_id,
                    session_id = session_id,
                    error = %e,
                    "failed to save recovered session messages"
                );
            }
        }
        Err(e) => {
            tracing::warn!(
                work_run_id = %job_id,
                error = %e,
                "failed to create message store"
            );
        }
    }
}

pub(super) fn cleanup_recovery(entry: &JournalEntry) {
    let env = IsolatedEnvironment {
        workdir: std::path::PathBuf::from(&entry.workdir),
        workspace_dir: std::path::PathBuf::from(&entry.workdir).join("workspace"),
        repos: Vec::new(),
        container_name: entry.container_name.clone(),
        secrets: HashMap::new(),
        env_vars: HashMap::new(),
        runtime: (entry.harness_type == "kata").then_some("kata-runtime"),
        image: Some(DEFAULT_IMAGE.to_owned()),
        server_host_port: None,
        limits: ResourceLimits::default(),
    };

    match entry.harness_type.as_str() {
        "host" => {
            kill_host_process_group(entry);
            tokio::spawn(async move {
                HostIsolation::new().cleanup(&env).await;
            });
        }
        "kata" => {
            tokio::spawn(async move {
                KataIsolation::new(DEFAULT_IMAGE.to_owned())
                    .cleanup(&env)
                    .await;
            });
        }
        _ => {
            tokio::spawn(async move {
                DockerIsolation::new(None, DEFAULT_IMAGE.to_owned())
                    .cleanup(&env)
                    .await;
            });
        }
    }
}

pub(crate) async fn mark_lost_and_submit(
    journal: &Arc<Journal>,
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    entry: &JournalEntry,
) {
    if let Err(e) = journal.update_result(JournalResultUpdate {
        job_id: entry.job_id,
        exit_code: 1,
        tokens_used: 0,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        pr_url: None,
        duration_ms: 0,
        review_url: None,
        review_body: None,
        review_already_exists: false,
        status: JournalStatus::Lost,
    }) {
        tracing::warn!(
            job_id = %entry.job_id,
            error = %e,
            "failed to mark stale job lost locally; skipping remote lost-result submission"
        );
        return;
    }

    let result = submit_result_request(SubmitResultParams {
        pr_urls: Vec::new(),
        exit_code: 1,
        tokens_used: 0,
        duration_ms: 0,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: None,
        result_summary: None,
        review_url: None,
        review_body: None,
        review_already_exists: false,
    });

    if let Err(e) = with_retry_on_401(client, worker_state, |token| {
        let client = client.clone();
        let result = result.clone();
        async move { client.submit_result(entry.job_id, &result, &token).await }
    })
    .await
    {
        if matches!(
            e.downcast_ref::<ApiError>(),
            Some(api_error) if api_error.status == 409
        ) {
            match journal.remove_job(entry.job_id) {
                Ok(()) => tracing::info!(
                    job_id = %entry.job_id,
                    "stale job was reset remotely; removed local recovery record"
                ),
                Err(remove_error) => tracing::warn!(
                    job_id = %entry.job_id,
                    error = %remove_error,
                    "failed to remove locally stale recovery record"
                ),
            }
            return;
        }
        tracing::warn!(
            job_id = %entry.job_id,
            error = %e,
            "failed to submit lost result for stale job"
        );
        return;
    }

    if let Err(e) = journal.mark_submitted(entry.job_id) {
        tracing::warn!(
            job_id = %entry.job_id,
            error = %e,
            "failed to mark stale job submitted locally"
        );
        return;
    }
    tracing::info!(job_id = %entry.job_id, "stale job marked as lost and submitted");
}
