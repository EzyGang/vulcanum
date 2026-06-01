use vulcanum_shared::api_types::SubmitResultRequest;
use vulcanum_shared::client::ApiClient;

use crate::runtime::runner::remove_container;
use crate::state::journal::{Journal, JournalEntry, JournalStatus};

pub async fn reconcile_running_jobs(journal: &Journal, client: &ApiClient, access_token: &str) {
    let running = match journal.list_running() {
        Ok(entries) => entries,
        Err(e) => {
            tracing::error!(error = %e, "failed to list running jobs for recovery");
            return;
        }
    };

    if running.is_empty() {
        return;
    }

    tracing::info!(count = running.len(), "reconciling stale running jobs");

    for entry in &running {
        let alive = check_container_alive(entry);

        if alive {
            let Some(name) = entry.container_name.as_deref() else {
                continue;
            };
            tracing::info!(
                job_id = %entry.job_id,
                container_name = name,
                "killing leftover container"
            );
            remove_container(Some(name));
        }

        let _ = journal.update_result(entry.job_id, 1, 0, None, 0, JournalStatus::Lost);

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

        if let Err(e) = client
            .submit_result(entry.job_id, &result, access_token)
            .await
        {
            tracing::warn!(
                job_id = %entry.job_id,
                error = %e,
                "failed to submit failed result for stale job"
            );
        }

        let _ = journal.mark_submitted(entry.job_id);
        tracing::info!(job_id = %entry.job_id, "stale job marked as lost and submitted");
    }
}

fn check_container_alive(entry: &JournalEntry) -> bool {
    let Some(name) = &entry.container_name else {
        return false;
    };

    let output = std::process::Command::new("docker")
        .args(["inspect", "--format", "{{.State.Running}}", name])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output();

    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim() == "true",
        Err(_) => false,
    }
}
