use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::project_configs::model::ProjectConfig;
use crate::models::provider_configs::model::IntegrationProvider;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRunStatus, WorkRunType};
use crate::services::work_runs::service::request_github_review::{
    GithubReviewRequest, GithubReviewRequestOutcome,
};
use crate::services::work_runs::service::review_ticket::{
    review_ticket_input, ReviewTicketCreator,
};
use crate::test_helpers;

const INSTALLATION_ID: i64 = 123;
const SENDER_ID: &str = "456";

#[derive(Default)]
struct MockReviewTicketCreator {
    created_count: AtomicUsize,
}

#[async_trait]
impl ReviewTicketCreator for MockReviewTicketCreator {
    async fn create(
        &self,
        _provider: &IntegrationProvider,
        _project: &ProjectConfig,
        repo_full_name: &str,
        pr_number: i64,
        _pr_title: &str,
    ) -> Result<String, WorkRunsError> {
        self.created_count.fetch_add(1, Ordering::SeqCst);
        Ok(format!("review-ticket-{repo_full_name}-{pr_number}"))
    }
}

#[sqlx::test]
async fn github_review_request_creates_standalone_review(pool: sqlx::PgPool) {
    let project_id = setup_review_project(&pool).await;
    let state = test_helpers::build_state(pool.clone()).await;
    let mut state = state;
    state.jobs = state
        .jobs
        .clone()
        .with_review_ticket_creator(Arc::new(MockReviewTicketCreator::default()));

    let project = state
        .jobs
        .project_configs
        .find_by_id(project_id)
        .await
        .expect("review project");
    let ticket_input = review_ticket_input(&project, "acme/widgets", 42, "Review me");
    assert_eq!(ticket_input.project_id, "github-review-project");
    assert_eq!(ticket_input.status, "in review");
    assert_eq!(ticket_input.title, "Review PR #42: Review me");
    assert_eq!(
        ticket_input.body,
        "Review pull request: https://github.com/acme/widgets/pull/42"
    );
    let outcome = state
        .jobs
        .request_github_review(GithubReviewRequest {
            delivery_id: "delivery-1",
            installation_id: INSTALLATION_ID,
            sender_id: SENDER_ID,
            repo_full_name: "Acme/Widgets",
            pr_number: 42,
            pr_title: "Review me",
            project_selector: None,
        })
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
    assert_eq!(run.external_task_ref, "review-ticket-acme/widgets-42");
    assert_eq!(run.task_title.as_deref(), Some("Review PR #42: Review me"));
    assert_eq!(run.task_slug.as_deref(), Some("Acme/Widgets#42"));
    assert_eq!(run.status, WorkRunStatus::Pending);
    assert_eq!(run.work_type, WorkRunType::PullRequestReview);
    assert_eq!(run.parent_work_run_id, None);
    assert_eq!(
        run.review_target_pr_url.as_deref(),
        Some("https://github.com/acme/widgets/pull/42")
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
    let creator = Arc::new(MockReviewTicketCreator::default());
    let mut state = state;
    state.jobs = state
        .jobs
        .clone()
        .with_review_ticket_creator(creator.clone());

    let unauthorized = state
        .jobs
        .request_github_review(GithubReviewRequest {
            delivery_id: "unauthorized",
            installation_id: INSTALLATION_ID,
            sender_id: "999",
            repo_full_name: "acme/widgets",
            pr_number: 42,
            pr_title: "Review me",
            project_selector: None,
        })
        .await
        .expect("reject unauthorized sender");
    assert_eq!(unauthorized, GithubReviewRequestOutcome::Unauthorized);

    let first = state
        .jobs
        .request_github_review(GithubReviewRequest {
            delivery_id: "delivery-1",
            installation_id: INSTALLATION_ID,
            sender_id: SENDER_ID,
            repo_full_name: "acme/widgets",
            pr_number: 42,
            pr_title: "Review me",
            project_selector: None,
        })
        .await
        .expect("create first review");
    let active_duplicate = state
        .jobs
        .request_github_review(GithubReviewRequest {
            delivery_id: "delivery-2",
            installation_id: INSTALLATION_ID,
            sender_id: SENDER_ID,
            repo_full_name: "Acme/Widgets",
            pr_number: 42,
            pr_title: "Review me again",
            project_selector: None,
        })
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
    assert_eq!(creator.created_count.load(Ordering::SeqCst), 1);
    let delivery_retry = state
        .jobs
        .request_github_review(GithubReviewRequest {
            delivery_id: "delivery-1",
            installation_id: INSTALLATION_ID,
            sender_id: SENDER_ID,
            repo_full_name: "acme/widgets",
            pr_number: 42,
            pr_title: "Review me",
            project_selector: None,
        })
        .await
        .expect("deduplicate delivery retry");
    let new_delivery = state
        .jobs
        .request_github_review(GithubReviewRequest {
            delivery_id: "delivery-3",
            installation_id: INSTALLATION_ID,
            sender_id: SENDER_ID,
            repo_full_name: "acme/widgets",
            pr_number: 42,
            pr_title: "Review latest head",
            project_selector: None,
        })
        .await
        .expect("create review for new delivery");
    assert_eq!(delivery_retry, GithubReviewRequestOutcome::AlreadyActive);
    assert_eq!(new_delivery, GithubReviewRequestOutcome::Spawned);
    assert_eq!(creator.created_count.load(Ordering::SeqCst), 1);
    let task_refs = sqlx::query_scalar!(
        "SELECT external_task_ref FROM work_runs WHERE review_target_pr_url = $1 ORDER BY created_at",
        "https://github.com/acme/widgets/pull/42",
    )
    .fetch_all(&pool)
    .await
    .expect("review task references");
    assert_eq!(task_refs, vec!["review-ticket-acme/widgets-42"; 2]);
}

#[sqlx::test]
async fn github_review_request_requires_deterministic_project_selection(pool: sqlx::PgPool) {
    let first_id = setup_review_project(&pool).await;
    let second_id = test_helpers::insert_project_config(&pool, "github-review-project-two").await;
    connect_repo(&pool, second_id).await;
    let state = test_helpers::build_state(pool.clone()).await;
    let mut state = state;
    state.jobs = state
        .jobs
        .clone()
        .with_review_ticket_creator(Arc::new(MockReviewTicketCreator::default()));

    let ambiguous = state
        .jobs
        .request_github_review(GithubReviewRequest {
            delivery_id: "ambiguous",
            installation_id: INSTALLATION_ID,
            sender_id: SENDER_ID,
            repo_full_name: "acme/widgets",
            pr_number: 42,
            pr_title: "Review me",
            project_selector: None,
        })
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
        .request_github_review(GithubReviewRequest {
            delivery_id: "selected",
            installation_id: INSTALLATION_ID,
            sender_id: SENDER_ID,
            repo_full_name: "acme/widgets",
            pr_number: 42,
            pr_title: "Review me",
            project_selector: Some(&format!("project:{second_id}")),
        })
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
    let mut state = state;
    state.jobs = state
        .jobs
        .clone()
        .with_review_ticket_creator(Arc::new(MockReviewTicketCreator::default()));

    assert!(matches!(
        state
            .jobs
            .request_github_review(GithubReviewRequest {
                delivery_id: "disabled",
                installation_id: INSTALLATION_ID,
                sender_id: SENDER_ID,
                repo_full_name: "acme/widgets",
                pr_number: 42,
                pr_title: "Review me",
                project_selector: Some(&format!("project:{project_id}")),
            })
            .await
            .expect("disabled outcome"),
        GithubReviewRequestOutcome::ReviewDisabled(_)
    ));
    assert!(matches!(
        state
            .jobs
            .request_github_review(GithubReviewRequest {
                delivery_id: "invalid",
                installation_id: INSTALLATION_ID,
                sender_id: SENDER_ID,
                repo_full_name: "acme/widgets",
                pr_number: 42,
                pr_title: "Review me",
                project_selector: Some("project:not-a-uuid"),
            })
            .await
            .expect("invalid outcome"),
        GithubReviewRequestOutcome::InvalidProjectSelection(_)
    ));
    assert!(matches!(
        state
            .jobs
            .request_github_review(GithubReviewRequest {
                delivery_id: "missing",
                installation_id: INSTALLATION_ID,
                sender_id: SENDER_ID,
                repo_full_name: "acme/other",
                pr_number: 42,
                pr_title: "Review me",
                project_selector: None,
            })
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
    let provider_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO integration_providers (id, team_id, name, instance_url, api_key) VALUES ($1, $2, $3, $4, $5)",
        provider_id,
        test_helpers::DEFAULT_TEAM_ID,
        "review-provider",
        "http://review-provider.invalid",
        "test-key",
    )
    .execute(pool)
    .await
    .expect("insert review provider");
    let project_id = test_helpers::insert_project_config_with_provider(
        pool,
        "github-review-project",
        provider_id,
    )
    .await;
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
    let provider_id = sqlx::query_scalar!(
        "SELECT id FROM integration_providers WHERE team_id = $1 ORDER BY created_at LIMIT 1",
        test_helpers::DEFAULT_TEAM_ID,
    )
    .fetch_one(pool)
    .await
    .expect("review provider");
    sqlx::query!(
        "UPDATE project_configs SET provider_id = $1 WHERE id = $2",
        provider_id,
        project_id,
    )
    .execute(pool)
    .await
    .expect("attach review provider");
}
