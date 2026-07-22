use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::Notify;
use uuid::Uuid;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::pull_requests::PullRequestCommentWriter;
use crate::services::github_app::service::webhooks::tests::processing::setup_review_request;
use crate::services::github_app::service::webhooks::tests::{service, service_with_writer};
use crate::services::github_app::webhook_store::{GithubWebhookDelivery, GithubWebhookKind};
use crate::test_helpers;

#[derive(Default)]
struct SlowWriter {
    calls: AtomicUsize,
    started: Notify,
    release: Notify,
}

#[async_trait]
impl PullRequestCommentWriter for SlowWriter {
    async fn ensure_pull_request_comment(
        &self,
        _team_id: Uuid,
        _installation_id: i64,
        _repo_full_name: &str,
        _pr_number: i64,
        _marker: &str,
        _body: &str,
    ) -> Result<(), GithubAppError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.started.notify_one();
        self.release.notified().await;
        Ok(())
    }
}

#[sqlx::test]
async fn active_delivery_lease_prevents_duplicate_slow_comment(pool: sqlx::PgPool) {
    setup_review_request(&pool).await;
    let state = test_helpers::build_state(pool).await;
    let writer = Arc::new(SlowWriter::default());
    let service = service_with_writer(&state, writer.clone());
    service
        .store
        .enqueue(GithubWebhookDelivery {
            delivery_id: "slow-comment".to_owned(),
            kind: GithubWebhookKind::ReviewRequested,
            installation_id: 123,
            repo_full_name: "acme/widgets".to_owned(),
            pr_number: 42,
            sender_id: Some("456".to_owned()),
            pr_title: Some("Review me".to_owned()),
            project_selector: Some("project:00000000-0000-0000-0000-000000000123".to_owned()),
            attempts: 0,
        })
        .await
        .expect("queue slow comment delivery");

    let first_service = service.clone();
    let first = tokio::spawn(async move { first_service.process_pending_once().await });
    tokio::time::timeout(Duration::from_secs(10), writer.started.notified())
        .await
        .expect("slow comment starts");
    tokio::time::sleep(Duration::from_millis(400)).await;

    assert!(!service
        .process_pending_once()
        .await
        .expect("active delivery is not reclaimed"));
    writer.release.notify_one();
    assert!(first
        .await
        .expect("first delivery worker exits")
        .expect("first delivery completes"));
    assert_eq!(writer.calls.load(Ordering::SeqCst), 1);
}

#[sqlx::test]
async fn stale_claim_cannot_complete_or_retry_new_owner(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool).await;
    let service = service(&state);
    let payload = test_helpers::github_webhook_payload("closed");
    let signature = test_helpers::sign_github_webhook(&payload);
    service
        .handle(&signature, "pull_request", "fenced-delivery", &payload)
        .await
        .expect("queue fenced delivery");
    let stale = service
        .store
        .claim_pending(Duration::from_millis(1))
        .await
        .expect("claim delivery")
        .expect("delivery exists");
    tokio::time::sleep(Duration::from_millis(5)).await;
    let current = service
        .store
        .claim_pending(Duration::from_secs(1))
        .await
        .expect("reclaim delivery")
        .expect("delivery is reclaimable");

    assert!(!service
        .store
        .complete(&stale)
        .await
        .expect("reject stale completion"));
    assert!(!service
        .store
        .retry(&stale, "stale")
        .await
        .expect("reject stale retry"));
    assert!(service
        .store
        .complete(&current)
        .await
        .expect("complete current claim"));
}
