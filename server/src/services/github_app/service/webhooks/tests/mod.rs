mod events;
mod leases;
mod processing;

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::pull_requests::PullRequestCommentWriter;
use crate::services::github_app::service::webhooks::GithubWebhookService;
use crate::services::github_app::webhook_store::GithubWebhookStore;
use crate::test_helpers;

const APP_SLUG: &str = "vulcanum-app";

#[derive(Default)]
struct RecordingWriter {
    calls: Mutex<Vec<(String, String)>>,
}

#[async_trait]
impl PullRequestCommentWriter for RecordingWriter {
    async fn ensure_pull_request_comment(
        &self,
        _team_id: Uuid,
        _installation_id: i64,
        _repo_full_name: &str,
        _pr_number: i64,
        marker: &str,
        body: &str,
    ) -> Result<(), GithubAppError> {
        self.calls
            .lock()
            .await
            .push((marker.to_owned(), body.to_owned()));
        Ok(())
    }
}

fn service(state: &crate::app_state::AppState) -> GithubWebhookService {
    service_with_writer(state, Arc::new(state.github.clone()))
}

fn service_with_writer(
    state: &crate::app_state::AppState,
    writer: Arc<dyn PullRequestCommentWriter>,
) -> GithubWebhookService {
    GithubWebhookService::new(
        Some(Arc::from(test_helpers::GITHUB_WEBHOOK_SECRET)),
        Some(Arc::from(APP_SLUG)),
        GithubWebhookStore::in_memory(),
        state.jobs.clone(),
        writer,
    )
}

fn issue_comment_payload(
    action: &str,
    state: &str,
    pull_request: Option<serde_json::Value>,
    body: &str,
    login: &str,
) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "action": action,
        "installation": {"id": 123},
        "repository": {"full_name": "acme/widgets"},
        "issue": {
            "number": 42,
            "title": "Review me",
            "state": state,
            "pull_request": pull_request,
        },
        "comment": {"body": body},
        "sender": {"id": 456, "login": login},
    }))
    .expect("serialize issue comment")
}
