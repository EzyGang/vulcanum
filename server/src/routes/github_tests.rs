use std::sync::Arc;

use actix_web::{http::StatusCode, test, web, App};

use crate::routes;
use crate::routes::github::{classify_callback, CallbackKind, CallbackQuery};
use crate::services::github_app::service::webhooks::GithubWebhookService;
use crate::services::github_app::webhook_store::GithubWebhookStore;
use crate::test_helpers;

#[actix_web::test]
async fn callback_classifier_accepts_oauth_response() {
    let query = CallbackQuery {
        code: Some("oauth-code".to_owned()),
        installation_id: None,
        setup_action: None,
        state: Some("oauth-state".to_owned()),
    };

    assert_eq!(
        classify_callback(&query),
        Some(CallbackKind::OAuth {
            code: "oauth-code",
            state_nonce: "oauth-state",
        })
    );
}

#[actix_web::test]
async fn callback_classifier_accepts_installation_response() {
    let query = CallbackQuery {
        code: None,
        installation_id: Some(42),
        setup_action: Some("install".to_owned()),
        state: Some("install-state".to_owned()),
    };

    assert_eq!(
        classify_callback(&query),
        Some(CallbackKind::Installation {
            installation_id: 42,
            setup_action: "install",
            state_nonce: "install-state",
        })
    );
}

#[sqlx::test]
async fn webhook_accepts_valid_delivery(pool: sqlx::PgPool) {
    let mut state = test_helpers::build_state(pool.clone()).await;
    state.github_webhooks = GithubWebhookService::new(
        Some(Arc::from(test_helpers::GITHUB_WEBHOOK_SECRET)),
        Some(Arc::from("vulcanum-app")),
        state.is_single_user,
        GithubWebhookStore::in_memory(),
        state.jobs.clone(),
        Arc::new(state.github.clone()),
    );
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;
    let payload = test_helpers::github_webhook_payload("closed");
    let request = test::TestRequest::post()
        .uri("/api/v1/github/webhook")
        .insert_header((
            "X-Hub-Signature-256",
            test_helpers::sign_github_webhook(&payload),
        ))
        .insert_header(("X-GitHub-Event", "pull_request"))
        .insert_header(("X-GitHub-Delivery", "delivery-1"))
        .set_payload(payload)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[sqlx::test]
async fn webhook_rejects_invalid_signature(pool: sqlx::PgPool) {
    let mut state = test_helpers::build_state(pool.clone()).await;
    state.github_webhooks = GithubWebhookService::new(
        Some(Arc::from(test_helpers::GITHUB_WEBHOOK_SECRET)),
        Some(Arc::from("vulcanum-app")),
        state.is_single_user,
        GithubWebhookStore::in_memory(),
        state.jobs.clone(),
        Arc::new(state.github.clone()),
    );
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
        .insert_header(("X-GitHub-Delivery", "delivery-invalid"))
        .set_payload(test_helpers::github_webhook_payload("closed"))
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn webhook_requires_delivery_identifier(pool: sqlx::PgPool) {
    let mut state = test_helpers::build_state(pool.clone()).await;
    state.github_webhooks = GithubWebhookService::new(
        Some(Arc::from(test_helpers::GITHUB_WEBHOOK_SECRET)),
        Some(Arc::from("vulcanum-app")),
        state.is_single_user,
        GithubWebhookStore::in_memory(),
        state.jobs.clone(),
        Arc::new(state.github.clone()),
    );
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;
    let payload = test_helpers::github_webhook_payload("closed");
    let request = test::TestRequest::post()
        .uri("/api/v1/github/webhook")
        .insert_header((
            "X-Hub-Signature-256",
            test_helpers::sign_github_webhook(&payload),
        ))
        .insert_header(("X-GitHub-Event", "pull_request"))
        .set_payload(payload)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
