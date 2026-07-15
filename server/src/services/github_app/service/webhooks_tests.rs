use std::sync::Arc;

use crate::services::github_app::service::webhooks::{
    verify_signature, GithubWebhookError, GithubWebhookOutcome, GithubWebhookService,
};
use crate::services::github_app::webhook_store::GithubWebhookStore;
use crate::test_helpers;

#[test]
fn signature_accepts_exact_payload() {
    let payload = test_helpers::github_webhook_payload("closed");
    let signature = test_helpers::sign_github_webhook(&payload);

    assert!(verify_signature(
        Some(test_helpers::GITHUB_WEBHOOK_SECRET),
        &signature,
        &payload,
    )
    .is_ok());
}

#[test]
fn signature_rejects_modified_payload() {
    let payload = test_helpers::github_webhook_payload("closed");
    let signature = test_helpers::sign_github_webhook(&payload);

    assert!(matches!(
        verify_signature(
            Some(test_helpers::GITHUB_WEBHOOK_SECRET),
            &signature,
            b"modified",
        ),
        Err(GithubWebhookError::InvalidSignature)
    ));
}

#[test]
fn signature_requires_configured_secret_and_sha256_format() {
    let payload = test_helpers::github_webhook_payload("closed");

    assert!(matches!(
        verify_signature(None, "sha256=00", &payload),
        Err(GithubWebhookError::NotConfigured)
    ));
    assert!(matches!(
        verify_signature(
            Some(test_helpers::GITHUB_WEBHOOK_SECRET),
            "sha1=00",
            &payload,
        ),
        Err(GithubWebhookError::InvalidSignature)
    ));
}

#[sqlx::test]
async fn completion_events_are_queued_idempotently(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool.clone()).await;
    let service = GithubWebhookService::new(
        Some(Arc::from(test_helpers::GITHUB_WEBHOOK_SECRET)),
        GithubWebhookStore::in_memory(),
        state.jobs,
    );
    let payload = test_helpers::github_webhook_payload("closed");
    let signature = test_helpers::sign_github_webhook(&payload);

    let first = service
        .handle(&signature, "pull_request", "delivery-1", &payload)
        .await
        .expect("queue delivery");
    let duplicate = service
        .handle(&signature, "pull_request", "delivery-1", &payload)
        .await
        .expect("accept duplicate delivery");

    assert_eq!(first, GithubWebhookOutcome::Queued { inserted: true });
    assert_eq!(duplicate, GithubWebhookOutcome::Queued { inserted: false });
}

#[sqlx::test]
async fn non_completion_events_are_ignored(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool.clone()).await;
    let service = GithubWebhookService::new(
        Some(Arc::from(test_helpers::GITHUB_WEBHOOK_SECRET)),
        GithubWebhookStore::in_memory(),
        state.jobs,
    );
    let payload = test_helpers::github_webhook_payload("opened");
    let signature = test_helpers::sign_github_webhook(&payload);

    let outcome = service
        .handle(&signature, "pull_request", "delivery-opened", &payload)
        .await
        .expect("ignore opened delivery");

    assert_eq!(outcome, GithubWebhookOutcome::Ignored);
    assert_eq!(
        service
            .handle(
                &test_helpers::sign_github_webhook(b"not-json"),
                "push",
                "delivery-push",
                b"not-json",
            )
            .await
            .expect("ignore unrelated event"),
        GithubWebhookOutcome::Ignored,
    );
}

#[sqlx::test]
async fn unmatched_close_delivery_remains_retryable(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool.clone()).await;
    let service = GithubWebhookService::new(
        Some(Arc::from(test_helpers::GITHUB_WEBHOOK_SECRET)),
        GithubWebhookStore::in_memory(),
        state.jobs,
    );
    let payload = test_helpers::github_webhook_payload("closed");
    let signature = test_helpers::sign_github_webhook(&payload);
    service
        .handle(&signature, "pull_request", "delivery-race", &payload)
        .await
        .expect("queue delivery");

    assert!(service
        .process_pending_once()
        .await
        .expect("process unmatched delivery"));

    tokio::time::sleep(std::time::Duration::from_millis(2_100)).await;
    assert!(service
        .process_pending_once()
        .await
        .expect("retry unmatched delivery"));
}
