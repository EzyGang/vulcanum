use actix_web::{http::StatusCode, test, web, App};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::routes;
use crate::services::github_app::service::webhooks::GithubWebhookService;
use crate::test_helpers;

const SECRET: &str = "route-webhook-secret";
const PAYLOAD: &[u8] = br#"{
    "action":"closed",
    "number":42,
    "installation":{"id":123},
    "repository":{"full_name":"acme/widgets"}
}"#;

#[sqlx::test]
async fn webhook_accepts_valid_signed_pull_request_event(pool: sqlx::PgPool) {
    let mut state = test_helpers::build_state(pool).await;
    state.github_webhooks = GithubWebhookService::new(Some(SECRET), state.jobs.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;
    let request = test::TestRequest::post()
        .uri("/api/v1/github/webhook")
        .insert_header(("X-Hub-Signature-256", sign(PAYLOAD)))
        .insert_header(("X-GitHub-Event", "pull_request"))
        .insert_header(("X-GitHub-Delivery", "delivery-1"))
        .set_payload(PAYLOAD)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[sqlx::test]
async fn webhook_rejects_invalid_signature(pool: sqlx::PgPool) {
    let mut state = test_helpers::build_state(pool).await;
    state.github_webhooks = GithubWebhookService::new(Some(SECRET), state.jobs.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;
    let request = test::TestRequest::post()
        .uri("/api/v1/github/webhook")
        .insert_header(("X-Hub-Signature-256", "sha256=00"))
        .insert_header(("X-GitHub-Event", "pull_request"))
        .set_payload(PAYLOAD)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

fn sign(body: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(SECRET.as_bytes()).expect("valid HMAC key");
    mac.update(body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}
