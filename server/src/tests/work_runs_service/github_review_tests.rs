use uuid::Uuid;

use crate::models::work_runs::model::{WorkRunStatus, WorkRunType};
use crate::services::work_runs::service::request_github_review::GithubReviewRequestOutcome;
use crate::test_helpers;

const INSTALLATION_ID: i64 = 123;
const SENDER_ID: &str = "456";

#[sqlx::test]
async fn github_review_request_creates_standalone_review(pool: sqlx::PgPool) {
    let project_id = setup_review_project(&pool).await;
    let state = test_helpers::build_state(pool.clone()).await;

    let outcome = state
        .jobs
        .request_github_review(
            "delivery-1",
            INSTALLATION_ID,
            SENDER_ID,
            "Acme/Widgets",
            42,
            "Review me",
            None,
        )
        .await
        .expect("request review");

    assert_eq!(outcome, GithubReviewRequestOutcome::Spawned);
    let run = sqlx::query!(
        r#"SELECT team_id, external_task_ref, task_title, task_slug, project_config_id,
           status as "status: WorkRunStatus", work_type as "work_type: WorkRunType",
           parent_work_run_id, review_target_pr_url, review_target_repo_full_name,
           github_installation_id, github_delivery_id
           FROM work_runs WHERE github_delivery_id = $1"#,
        "delivery-1",
    )
    .fetch_one(&pool)
    .await
    .expect("standalone review row");
    assert_eq!(run.team_id, test_helpers::DEFAULT_TEAM_ID);
    assert_eq!(run.project_config_id, project_id);
    assert_eq!(run.external_task_ref, "github-pr:acme/widgets#42");
    assert_eq!(run.task_title.as_deref(), Some("Review me"));
    assert_eq!(run.task_slug.as_deref(), Some("Acme/Widgets#42"));
    assert_eq!(run.status, WorkRunStatus::Pending);
    assert_eq!(run.work_type, WorkRunType::PullRequestReview);
    assert_eq!(run.parent_work_run_id, None);
    assert_eq!(
        run.review_target_pr_url.as_deref(),
        Some("https://github.com/Acme/Widgets/pull/42")
    );
    assert_eq!(
        run.review_target_repo_full_name.as_deref(),
        Some("Acme/Widgets")
    );
    assert_eq!(run.github_installation_id, Some(INSTALLATION_ID));
}

#[sqlx::test]
async fn github_review_request_is_authorized_and_idempotent(pool: sqlx::PgPool) {
    setup_review_project(&pool).await;
    let state = test_helpers::build_state(pool.clone()).await;

    let unauthorized = state
        .jobs
        .request_github_review(
            "unauthorized",
            INSTALLATION_ID,
            "999",
            "acme/widgets",
            42,
            "Review me",
            None,
        )
        .await
        .expect("reject unauthorized sender");
    assert_eq!(unauthorized, GithubReviewRequestOutcome::Unauthorized);

    let first = state
        .jobs
        .request_github_review(
            "delivery-1",
            INSTALLATION_ID,
            SENDER_ID,
            "acme/widgets",
            42,
            "Review me",
            None,
        )
        .await
        .expect("create first review");
    let active_duplicate = state
        .jobs
        .request_github_review(
            "delivery-2",
            INSTALLATION_ID,
            SENDER_ID,
            "acme/widgets",
            42,
            "Review me again",
            None,
        )
        .await
        .expect("deduplicate active review");
    assert_eq!(first, GithubReviewRequestOutcome::Spawned);
    assert_eq!(active_duplicate, GithubReviewRequestOutcome::AlreadyActive);

    sqlx::query!(
        "UPDATE work_runs SET status = 'completed'::work_run_status WHERE github_delivery_id = $1",
        "delivery-1",
    )
    .execute(&pool)
    .await
    .expect("complete first review");
    let delivery_retry = state
        .jobs
        .request_github_review(
            "delivery-1",
            INSTALLATION_ID,
            SENDER_ID,
            "acme/widgets",
            42,
            "Review me",
            None,
        )
        .await
        .expect("deduplicate delivery retry");
    let new_delivery = state
        .jobs
        .request_github_review(
            "delivery-3",
            INSTALLATION_ID,
            SENDER_ID,
            "acme/widgets",
            42,
            "Review latest head",
            None,
        )
        .await
        .expect("create review for new delivery");
    assert_eq!(delivery_retry, GithubReviewRequestOutcome::AlreadyActive);
    assert_eq!(new_delivery, GithubReviewRequestOutcome::Spawned);
}

#[sqlx::test]
async fn github_review_request_requires_deterministic_project_selection(pool: sqlx::PgPool) {
    let first_id = setup_review_project(&pool).await;
    let second_id = test_helpers::insert_project_config(&pool, "github-review-project-two").await;
    connect_repo(&pool, second_id).await;
    let state = test_helpers::build_state(pool.clone()).await;

    let ambiguous = state
        .jobs
        .request_github_review(
            "ambiguous",
            INSTALLATION_ID,
            SENDER_ID,
            "acme/widgets",
            42,
            "Review me",
            None,
        )
        .await
        .expect("require project selection");
    match ambiguous {
        GithubReviewRequestOutcome::ProjectSelectionRequired(options) => {
            assert_eq!(options.projects.len(), 2);
            assert_eq!(options.projects[0].project_config_id, first_id);
            assert_eq!(options.projects[1].project_config_id, second_id);
        }
        outcome => panic!("unexpected outcome: {outcome:?}"),
    }

    let selected = state
        .jobs
        .request_github_review(
            "selected",
            INSTALLATION_ID,
            SENDER_ID,
            "acme/widgets",
            42,
            "Review me",
            Some(&format!("project:{second_id}")),
        )
        .await
        .expect("select project");
    assert_eq!(selected, GithubReviewRequestOutcome::Spawned);
    let selected_project = sqlx::query_scalar!(
        "SELECT project_config_id FROM work_runs WHERE github_delivery_id = $1",
        "selected",
    )
    .fetch_one(&pool)
    .await
    .expect("selected review row");
    assert_eq!(selected_project, second_id);
}

#[sqlx::test]
async fn github_review_request_explains_disabled_invalid_and_missing_projects(pool: sqlx::PgPool) {
    let project_id = setup_review_project(&pool).await;
    sqlx::query!(
        "UPDATE project_configs SET review_enabled = false WHERE id = $1",
        project_id,
    )
    .execute(&pool)
    .await
    .expect("disable project review");
    let state = test_helpers::build_state(pool).await;

    assert!(matches!(
        state
            .jobs
            .request_github_review(
                "disabled",
                INSTALLATION_ID,
                SENDER_ID,
                "acme/widgets",
                42,
                "Review me",
                Some(&format!("project:{project_id}")),
            )
            .await
            .expect("disabled outcome"),
        GithubReviewRequestOutcome::ReviewDisabled(_)
    ));
    assert!(matches!(
        state
            .jobs
            .request_github_review(
                "invalid",
                INSTALLATION_ID,
                SENDER_ID,
                "acme/widgets",
                42,
                "Review me",
                Some("project:not-a-uuid"),
            )
            .await
            .expect("invalid outcome"),
        GithubReviewRequestOutcome::InvalidProjectSelection(_)
    ));
    assert!(matches!(
        state
            .jobs
            .request_github_review(
                "missing",
                INSTALLATION_ID,
                SENDER_ID,
                "acme/other",
                42,
                "Review me",
                None,
            )
            .await
            .expect("missing outcome"),
        GithubReviewRequestOutcome::NoMatchingProject { .. }
    ));
}

async fn setup_review_project(pool: &sqlx::PgPool) -> Uuid {
    test_helpers::ensure_default_team(pool).await;
    sqlx::query!(
        "UPDATE teams SET review_enabled = true WHERE id = $1",
        test_helpers::DEFAULT_TEAM_ID,
    )
    .execute(pool)
    .await
    .expect("enable team reviews");
    let project_id = test_helpers::insert_project_config(pool, "github-review-project").await;
    sqlx::query!(
        "INSERT INTO project_config_repos (project_config_id, repo_full_name, repo_url, position) VALUES ($1, $2, $3, 0)",
        project_id,
        "acme/widgets",
        "https://github.com/acme/widgets",
    )
    .execute(pool)
    .await
    .expect("connect repository");
    sqlx::query!(
        "INSERT INTO github_installations (github_installation_id, account_login, team_id) VALUES ($1, $2, $3)",
        INSTALLATION_ID,
        "acme",
        test_helpers::DEFAULT_TEAM_ID,
    )
    .execute(pool)
    .await
    .expect("connect installation");
    sqlx::query!(
        "INSERT INTO users (id, email) VALUES ($1, $2)",
        "github-review-user",
        "github-review@example.com",
    )
    .execute(pool)
    .await
    .expect("insert user");

    sqlx::query!(
        "INSERT INTO team_members (team_id, user_id) VALUES ($1, $2)",
        test_helpers::DEFAULT_TEAM_ID,
        "github-review-user",
    )
    .execute(pool)
    .await
    .expect("insert membership");
    sqlx::query!(
        "INSERT INTO user_identities (id, user_id, provider, provider_user_id, provider_login, provider_verified_at) VALUES ($1, $2, 'github', $3, $4, NOW())",
        Uuid::new_v4(),
        "github-review-user",
        SENDER_ID,
        "octocat",
    )
    .execute(pool)
    .await
    .expect("insert verified identity");
    project_id
}

async fn connect_repo(pool: &sqlx::PgPool, project_id: Uuid) {
    sqlx::query!(
        "INSERT INTO project_config_repos (project_config_id, repo_full_name, repo_url, position) VALUES ($1, $2, $3, 0)",
        project_id,
        "acme/widgets",
        "https://github.com/acme/widgets",
    )
    .execute(pool)
    .await
    .expect("connect repository");
}
