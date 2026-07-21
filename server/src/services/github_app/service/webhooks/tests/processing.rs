use std::time::Duration;

use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::pull_requests::PullRequestCommentWriter;
use crate::services::github_app::service::webhooks::responses::respond_to_outcome;
use crate::services::github_app::service::webhooks::tests::{
    issue_comment_payload, service, RecordingWriter, APP_SLUG,
};
use crate::services::work_runs::service::request_github_review::{
    GithubReviewRequestOutcome, ReviewProjectOption, ReviewResponseOptions,
};
use crate::test_helpers;

#[sqlx::test]
async fn signed_review_command_creates_standalone_run(pool: sqlx::PgPool) {
    setup_review_request(&pool).await;
    let state = test_helpers::build_state(pool.clone()).await;
    let service = service(&state);
    let payload = issue_comment_payload(
        "created",
        "open",
        Some(serde_json::json!({})),
        "@vulcanum-app review",
        "octocat",
    );
    let signature = test_helpers::sign_github_webhook(&payload);

    service
        .handle(&signature, "issue_comment", "smoke-delivery", &payload)
        .await
        .expect("queue signed review command");
    assert!(service
        .process_pending_once()
        .await
        .expect("process signed review command"));

    let run = sqlx::query!(
        "SELECT parent_work_run_id, github_installation_id, github_delivery_id FROM work_runs WHERE github_delivery_id = $1",
        "smoke-delivery",
    )
    .fetch_one(&pool)
    .await
    .expect("standalone review run");
    assert_eq!(run.parent_work_run_id, None);
    assert_eq!(run.github_installation_id, Some(123));
}

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

#[sqlx::test]
async fn comment_writer_rejects_disconnected_installation_before_github_call(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool).await;
    let error = state
        .github
        .ensure_pull_request_comment(
            test_helpers::DEFAULT_TEAM_ID,
            999,
            "acme/widgets",
            42,
            "<!-- marker -->",
            "body",
        )
        .await
        .expect_err("disconnected installation must be rejected");
    assert!(matches!(error, GithubAppError::NoInstallation));
}

#[tokio::test]
async fn selection_reply_contains_marker_and_exact_commands() {
    let writer = RecordingWriter::default();
    let first_id = Uuid::new_v4();
    let second_id = Uuid::new_v4();
    let outcome = GithubReviewRequestOutcome::ProjectSelectionRequired(ReviewResponseOptions {
        team_id: Uuid::new_v4(),
        projects: vec![
            ReviewProjectOption {
                project_config_id: first_id,
                display_name: "First project".to_owned(),
            },
            ReviewProjectOption {
                project_config_id: second_id,
                display_name: "Second project".to_owned(),
            },
        ],
    });
    respond_to_outcome(
        &writer,
        APP_SLUG,
        "delivery-choice",
        123,
        "acme/widgets",
        42,
        &outcome,
    )
    .await
    .expect("write selection reply");
    let calls = writer.calls.lock().await;
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0].0,
        "<!-- vulcanum:github-delivery:delivery-choice -->"
    );
    assert!(calls[0]
        .1
        .contains(&format!("@{APP_SLUG} review project:{first_id}")));
    assert!(calls[0]
        .1
        .contains(&format!("@{APP_SLUG} review project:{second_id}")));
}

async fn setup_review_request(pool: &sqlx::PgPool) {
    test_helpers::ensure_default_team(pool).await;
    sqlx::query!(
        "UPDATE teams SET review_enabled = true WHERE id = $1",
        test_helpers::DEFAULT_TEAM_ID,
    )
    .execute(pool)
    .await
    .expect("enable reviews");
    let project_id = test_helpers::insert_project_config(pool, "webhook-smoke").await;
    sqlx::query!(
        "INSERT INTO project_config_repos (project_config_id, repo_full_name, repo_url, position) VALUES ($1, 'acme/widgets', 'https://github.com/acme/widgets', 0)",
        project_id,
    )
    .execute(pool)
    .await
    .expect("connect repo");
    sqlx::query!(
        "INSERT INTO github_installations (github_installation_id, account_login, team_id) VALUES (123, 'acme', $1)",
        test_helpers::DEFAULT_TEAM_ID,
    )
    .execute(pool)
    .await
    .expect("connect installation");
    sqlx::query!("INSERT INTO users (id, email) VALUES ('webhook-user', 'webhook@example.com')",)
        .execute(pool)
        .await
        .expect("insert user");
    sqlx::query!(
        "INSERT INTO team_members (team_id, user_id) VALUES ($1, 'webhook-user')",
        test_helpers::DEFAULT_TEAM_ID,
    )
    .execute(pool)
    .await
    .expect("insert membership");
    sqlx::query!(
        "INSERT INTO user_identities (id, user_id, provider, provider_user_id, provider_login, provider_verified_at) VALUES ($1, 'webhook-user', 'github', '456', 'octocat', NOW())",
        Uuid::new_v4(),
    )
    .execute(pool)
    .await
    .expect("insert identity");
}
