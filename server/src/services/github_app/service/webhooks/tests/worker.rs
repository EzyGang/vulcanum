use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::services::github_app::service::webhooks::tests::service;
use crate::test_helpers;

#[sqlx::test]
async fn unmatched_close_delivery_remains_retryable(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool).await;
    let service = service(&state);
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
    tokio::time::sleep(Duration::from_millis(2_100)).await;
    assert!(service
        .process_pending_once()
        .await
        .expect("retry unmatched delivery"));
}

#[sqlx::test]
async fn worker_stops_when_cancelled(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool).await;
    let service = service(&state);
    let cancellation = CancellationToken::new();
    let worker = tokio::spawn(service.run(cancellation.child_token()));
    cancellation.cancel();
    tokio::time::timeout(Duration::from_secs(1), worker)
        .await
        .expect("worker observes cancellation")
        .expect("worker exits cleanly");
}
