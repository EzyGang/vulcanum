use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use vulcanum_shared::api_types::SubmitResultRequest;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::runtime::types::{FinishRunArtifact, FinishStatus, SessionExport};
use vulcanum_shared::worker_state::WorkerState;

use crate::daemon::auth::with_retry_on_401;
use crate::state::journal::{Journal, JournalResultUpdate, JournalStatus};

pub(crate) struct FailedResult {
    pub(crate) exit_code: i32,
    pub(crate) tokens_used: i64,
    pub(crate) pr_urls: Vec<String>,
    pub(crate) duration_ms: i64,
    pub(crate) finish_status: Option<FinishStatus>,
    pub(crate) finish_summary: Option<String>,
    pub(crate) finish_blocked_reason: Option<String>,
    pub(crate) finish_next_column: Option<String>,
    pub(crate) review_url: Option<String>,
    pub(crate) review_body: Option<String>,
    pub(crate) review_already_exists: bool,
}

impl FailedResult {
    #[must_use]
    pub(crate) fn empty() -> Self {
        Self {
            exit_code: 1,
            tokens_used: 0,
            pr_urls: Vec::new(),
            duration_ms: 0,
            finish_status: None,
            finish_summary: None,
            finish_blocked_reason: None,
            finish_next_column: None,
            review_url: None,
            review_body: None,
            review_already_exists: false,
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
    let _ = journal.update_result(JournalResultUpdate {
        job_id,
        exit_code: result.exit_code,
        tokens_used: result.tokens_used,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        pr_url: result.pr_urls.first().map(String::as_str),
        duration_ms: result.duration_ms,
        status: JournalStatus::Failed,
    });
    let submit = submit_result_request(SubmitResultParams {
        pr_urls: result.pr_urls.clone(),
        exit_code: result.exit_code,
        tokens_used: result.tokens_used,
        duration_ms: result.duration_ms,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: result.finish_status,
        finish_summary: result.finish_summary.clone(),
        finish_blocked_reason: result.finish_blocked_reason.clone(),
        finish_next_column: result.finish_next_column.clone(),
        review_url: result.review_url.clone(),
        review_body: result.review_body.clone(),
        review_already_exists: result.review_already_exists,
    });
    if let Err(e) = with_retry_on_401(&client, &worker_state, |token| {
        let client = client.clone();
        let submit = submit.clone();
        async move { client.submit_result(job_id, &submit, &token).await }
    })
    .await
    {
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

    let pr_urls = finish_artifact.map(artifact_pr_urls).unwrap_or_default();
    let pr_url = pr_urls.first().map(String::as_str);

    let _ = journal.update_result(JournalResultUpdate {
        job_id,
        exit_code: session_export.exit_code,
        tokens_used: to_i64_saturating(session_export.tokens_used),
        input_tokens: to_i64_saturating(session_export.input_tokens),
        output_tokens: to_i64_saturating(session_export.output_tokens),
        cache_read_tokens: to_i64_saturating(session_export.cache_read_tokens),
        cache_write_tokens: to_i64_saturating(session_export.cache_write_tokens),
        pr_url,
        duration_ms: to_i64_saturating(session_export.duration_ms),
        status: journal_status,
    });

    let result = submit_result_request(SubmitResultParams {
        pr_urls,
        exit_code: session_export.exit_code,
        tokens_used: to_i64_saturating(session_export.tokens_used),
        duration_ms: to_i64_saturating(session_export.duration_ms),
        input_tokens: to_i64_saturating(session_export.input_tokens),
        output_tokens: to_i64_saturating(session_export.output_tokens),
        cache_read_tokens: to_i64_saturating(session_export.cache_read_tokens),
        cache_write_tokens: to_i64_saturating(session_export.cache_write_tokens),
        model_used: session_export.model_used.clone(),
        finish_status: finish_artifact.map(|a| a.status),
        finish_summary: finish_artifact.and_then(|a| a.summary.clone()),
        finish_blocked_reason: finish_artifact.and_then(|a| a.blocked_reason.clone()),
        finish_next_column: finish_artifact.and_then(|a| a.next_column.clone()),
        review_url: finish_artifact.and_then(|a| a.review_url.clone()),
        review_body: finish_artifact.and_then(|a| a.review_body.clone()),
        review_already_exists: finish_artifact
            .map(|a| a.review_already_exists)
            .unwrap_or(false),
    });

    if let Err(e) = with_retry_on_401(client, worker_state, |token| {
        let client = client.clone();
        let result = result.clone();
        async move { client.submit_result(job_id, &result, &token).await }
    })
    .await
    {
        tracing::error!(
            work_run_id = %job_id,
            error = %e,
            "submit_result failed for job",
        );
    }
    let _ = journal.mark_submitted(job_id);
}

pub(crate) struct SubmitResultParams {
    pub(crate) pr_urls: Vec<String>,
    pub(crate) exit_code: i32,
    pub(crate) tokens_used: i64,
    pub(crate) duration_ms: i64,
    pub(crate) input_tokens: i64,
    pub(crate) output_tokens: i64,
    pub(crate) cache_read_tokens: i64,
    pub(crate) cache_write_tokens: i64,
    pub(crate) model_used: Option<String>,
    pub(crate) finish_status: Option<FinishStatus>,
    pub(crate) finish_summary: Option<String>,
    pub(crate) finish_blocked_reason: Option<String>,
    pub(crate) finish_next_column: Option<String>,
    pub(crate) review_url: Option<String>,
    pub(crate) review_body: Option<String>,
    pub(crate) review_already_exists: bool,
}

#[must_use]
pub(crate) fn submit_result_request(params: SubmitResultParams) -> SubmitResultRequest {
    SubmitResultRequest {
        pr_url: params.pr_urls.first().cloned().unwrap_or_default(),
        pr_urls: params.pr_urls,
        exit_code: params.exit_code,
        tokens_used: params.tokens_used,
        duration_ms: params.duration_ms,
        input_tokens: params.input_tokens,
        output_tokens: params.output_tokens,
        cache_read_tokens: params.cache_read_tokens,
        cache_write_tokens: params.cache_write_tokens,
        model_used: params.model_used,
        finish_status: params.finish_status,
        finish_summary: params.finish_summary,
        finish_blocked_reason: params.finish_blocked_reason,
        finish_next_column: params.finish_next_column,
        review_url: params.review_url,
        review_body: params.review_body,
        review_already_exists: params.review_already_exists,
    }
}

#[must_use]
fn artifact_pr_urls(artifact: &FinishRunArtifact) -> Vec<String> {
    if !artifact.pr_urls.is_empty() {
        return artifact.pr_urls.clone();
    }
    single_url_to_vec(artifact.pr_url.clone())
}

#[must_use]
fn single_url_to_vec(url: Option<String>) -> Vec<String> {
    url.map(|url| vec![url]).unwrap_or_default()
}

#[must_use]
fn to_i64_saturating(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
