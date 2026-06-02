use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use vulcanum_shared::api_types::SubmitResultRequest;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::runtime::types::{FinishRunArtifact, FinishStatus, SessionExport};
use vulcanum_shared::worker_state::WorkerState;

use crate::state::journal::{Journal, JournalStatus};

pub(crate) struct FailedResult {
    pub(crate) exit_code: i32,
    pub(crate) tokens_used: i64,
    pub(crate) pr_url: Option<String>,
    pub(crate) duration_ms: i64,
    pub(crate) finish_status: Option<FinishStatus>,
    pub(crate) finish_summary: Option<String>,
    pub(crate) finish_blocked_reason: Option<String>,
    pub(crate) finish_next_column: Option<String>,
}

impl FailedResult {
    pub(crate) fn empty() -> Self {
        Self {
            exit_code: 1,
            tokens_used: 0,
            pr_url: None,
            duration_ms: 0,
            finish_status: None,
            finish_summary: None,
            finish_blocked_reason: None,
            finish_next_column: None,
        }
    }
}

pub(crate) async fn submit_failed_result(
    client: Arc<ApiClient>,
    worker_state: Arc<RwLock<WorkerState>>,
    journal: Arc<Journal>,
    job_id: Uuid,
    result: &FailedResult,
) {
    let _ = journal.update_result(
        job_id,
        result.exit_code,
        result.tokens_used,
        result.pr_url.as_deref(),
        result.duration_ms,
        JournalStatus::Failed,
    );
    let submit = SubmitResultRequest {
        pr_url: result.pr_url.clone().unwrap_or_default(),
        exit_code: result.exit_code,
        tokens_used: result.tokens_used,
        duration_ms: result.duration_ms,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: result.finish_status.clone(),
        finish_summary: result.finish_summary.clone(),
        finish_blocked_reason: result.finish_blocked_reason.clone(),
        finish_next_column: result.finish_next_column.clone(),
    };
    let access_token = worker_state.read().await.access_token.clone();
    if let Err(e) = client.submit_result(job_id, &submit, &access_token).await {
        tracing::error!(work_run_id = %job_id, error = %e, "submit_result failed for job");
    }
    let _ = journal.mark_submitted(job_id);
}

pub(crate) async fn submit_turn_result(
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    journal: &Arc<Journal>,
    job_id: Uuid,
    session_export: &SessionExport,
    finish_artifact: Option<&FinishRunArtifact>,
) {
    let journal_status = match session_export.exit_code {
        0 => JournalStatus::Completed,
        _ => JournalStatus::Failed,
    };

    let _ = journal.update_result(
        job_id,
        session_export.exit_code,
        session_export.tokens_used as i64,
        session_export.pr_url.as_deref(),
        session_export.duration_ms as i64,
        journal_status,
    );

    let result = SubmitResultRequest {
        pr_url: session_export.pr_url.clone().unwrap_or_default(),
        exit_code: session_export.exit_code,
        tokens_used: session_export.tokens_used as i64,
        duration_ms: session_export.duration_ms as i64,
        input_tokens: session_export.input_tokens as i64,
        output_tokens: session_export.output_tokens as i64,
        cache_read_tokens: session_export.cache_read_tokens as i64,
        cache_write_tokens: session_export.cache_write_tokens as i64,
        model_used: session_export.model_used.clone(),
        finish_status: finish_artifact.map(|a| a.status.clone()),
        finish_summary: finish_artifact.and_then(|a| a.summary.clone()),
        finish_blocked_reason: finish_artifact.and_then(|a| a.blocked_reason.clone()),
        finish_next_column: finish_artifact.and_then(|a| a.next_column.clone()),
    };

    let access_token = worker_state.read().await.access_token.clone();
    if let Err(e) = client.submit_result(job_id, &result, &access_token).await {
        tracing::error!(
            work_run_id = %job_id,
            error = %e,
            "submit_result failed for job",
        );
    }
    let _ = journal.mark_submitted(job_id);
}

pub(crate) fn read_finish_artifact(path: &std::path::Path) -> Option<FinishRunArtifact> {
    let raw = std::fs::read_to_string(path).ok()?;
    match serde_json::from_str::<FinishRunArtifact>(&raw) {
        Ok(artifact) => {
            tracing::info!(status = ?artifact.status, "parsed finish artifact");
            Some(artifact)
        }
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "failed to parse finish artifact");
            None
        }
    }
}
