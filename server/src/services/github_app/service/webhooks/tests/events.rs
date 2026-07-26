use std::sync::Arc;
use std::time::Duration;

use crate::services::github_app::service::webhooks::events::parse_event;
use crate::services::github_app::service::webhooks::tests::{issue_comment_payload, service};
use crate::services::github_app::service::webhooks::{
    verify_signature, GithubWebhookError, GithubWebhookOutcome, GithubWebhookService,
};
use crate::services::github_app::webhook_store::{
    GithubWebhookCommandError, GithubWebhookKind, GithubWebhookStore,
};
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

#[sqlx::test]
async fn completion_events_are_queued_idempotently(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool).await;
    let service = service(&state);
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
async fn issue_comment_command_is_queued_with_review_fields(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool).await;
    let service = service(&state);
    let payload = issue_comment_payload(
        "created",
        "open",
        Some(serde_json::json!({"url": "https://api.github.com/pulls/42"})),
        "Please @Vulcanum-App review project:00000000-0000-0000-0000-000000000123",
        "octocat",
    );
    let signature = test_helpers::sign_github_webhook(&payload);
    assert_eq!(
        service
            .handle(&signature, "issue_comment", "delivery-review", &payload)
            .await
            .expect("queue review request"),
        GithubWebhookOutcome::Queued { inserted: true },
    );
    let queued = service
        .store
        .claim_pending(Duration::from_secs(60))
        .await
        .expect("claim queued request")
        .expect("request exists");
    assert_eq!(queued.delivery.kind, GithubWebhookKind::ReviewRequested);
    assert_eq!(queued.delivery.comment_id, Some(789));
    assert_eq!(queued.delivery.sender_id.as_deref(), Some("456"));
    assert_eq!(queued.delivery.pr_title.as_deref(), Some("Review me"));
    assert_eq!(
        queued.delivery.project_selector.as_deref(),
        Some("project:00000000-0000-0000-0000-000000000123")
    );
    assert_eq!(queued.delivery.request_body, None);
    assert_eq!(queued.delivery.command_error, None);
}

#[test]
fn implement_command_preserves_multiline_request_and_optional_selector() {
    let request = "Handle the retry case.\n\nAlso preserve this `exact` section.";
    let without_selector = parse_issue_comment(&format!("@VULCANUM-APP ImPlEmEnT {request}"));
    assert_eq!(
        without_selector.kind,
        GithubWebhookKind::ImplementationFollowupRequested
    );
    assert_eq!(without_selector.project_selector, None);
    assert_eq!(without_selector.request_body.as_deref(), Some(request));
    assert_eq!(without_selector.command_error, None);

    let project_id = "00000000-0000-0000-0000-000000000123";
    let with_selector = parse_issue_comment(&format!(
        "@vulcanum-app implement project:{project_id} {request}"
    ));
    assert_eq!(
        with_selector.project_selector.as_deref(),
        Some("project:00000000-0000-0000-0000-000000000123")
    );
    assert_eq!(with_selector.request_body.as_deref(), Some(request));
}

#[test]
fn malformed_and_ambiguous_implement_commands_are_queued_for_feedback() {
    for command in [
        "@vulcanum-app implement",
        "@vulcanum-app implement project:00000000-0000-0000-0000-000000000123",
    ] {
        let delivery = parse_issue_comment(command);
        assert_eq!(
            delivery.kind,
            GithubWebhookKind::ImplementationFollowupRequested
        );
        assert_eq!(
            delivery.command_error,
            Some(GithubWebhookCommandError::Malformed)
        );
        assert_eq!(delivery.request_body, None);
    }

    let delivery =
        parse_issue_comment("@vulcanum-app review\n@vulcanum-app implement handle retries");
    assert_eq!(
        delivery.command_error,
        Some(GithubWebhookCommandError::Ambiguous)
    );
    assert_eq!(
        delivery.kind,
        GithubWebhookKind::ImplementationFollowupRequested
    );
}

#[sqlx::test]
async fn invalid_issue_comment_shapes_are_ignored(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool).await;
    let service = service(&state);
    let cases = [
        issue_comment_payload(
            "edited",
            "open",
            Some(serde_json::json!({})),
            "@vulcanum-app review",
            "octocat",
        ),
        issue_comment_payload(
            "deleted",
            "open",
            Some(serde_json::json!({})),
            "@vulcanum-app implement handle retries",
            "octocat",
        ),
        issue_comment_payload(
            "created",
            "closed",
            Some(serde_json::json!({})),
            "@vulcanum-app review",
            "octocat",
        ),
        issue_comment_payload("created", "open", None, "@vulcanum-app review", "octocat"),
        issue_comment_payload(
            "created",
            "open",
            Some(serde_json::json!({})),
            "@vulcanum-app-extra review",
            "octocat",
        ),
        issue_comment_payload(
            "created",
            "open",
            Some(serde_json::json!({})),
            "no mention",
            "octocat",
        ),
        issue_comment_payload(
            "created",
            "open",
            Some(serde_json::json!({})),
            "@vulcanum-app review",
            "vulcanum-app[bot]",
        ),
        issue_comment_payload(
            "created",
            "open",
            Some(serde_json::json!({})),
            "@vulcanum-app can you help?",
            "octocat",
        ),
        issue_comment_payload(
            "created",
            "open",
            Some(serde_json::json!({})),
            "@vulcanum-app review later",
            "octocat",
        ),
    ];
    for (index, payload) in cases.iter().enumerate() {
        let signature = test_helpers::sign_github_webhook(payload);
        assert_eq!(
            service
                .handle(
                    &signature,
                    "issue_comment",
                    &format!("ignored-{index}"),
                    payload,
                )
                .await
                .expect("ignore invalid command"),
            GithubWebhookOutcome::Ignored,
        );
    }
}

fn parse_issue_comment(
    command: &str,
) -> crate::services::github_app::webhook_store::GithubWebhookDelivery {
    let payload = issue_comment_payload(
        "created",
        "open",
        Some(serde_json::json!({})),
        command,
        "octocat",
    );
    parse_event(
        "issue_comment",
        "parser-delivery",
        Some("vulcanum-app"),
        &payload,
    )
    .expect("parse issue comment")
    .expect("queue command")
}

#[sqlx::test]
async fn issue_comment_requires_app_slug(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool).await;
    let service = GithubWebhookService::new(
        Some(Arc::from(test_helpers::GITHUB_WEBHOOK_SECRET)),
        None,
        state.is_single_user,
        GithubWebhookStore::in_memory(),
        state.jobs.clone(),
        Arc::new(state.github.clone()),
    );
    let payload = issue_comment_payload(
        "created",
        "open",
        Some(serde_json::json!({})),
        "@app",
        "octocat",
    );
    let signature = test_helpers::sign_github_webhook(&payload);
    assert!(matches!(
        service
            .handle(&signature, "issue_comment", "missing-slug", &payload)
            .await,
        Err(GithubWebhookError::MissingAppSlug)
    ));
}
