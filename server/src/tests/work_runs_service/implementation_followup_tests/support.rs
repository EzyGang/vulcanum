use std::sync::Arc;

use uuid::Uuid;

use crate::services::work_runs::service::request_github_implementation::GithubImplementationRequest;
use crate::services::work_runs::service::WorkRunsService;
use crate::test_helpers;
use crate::tests::work_runs_service::implementation_followup_tests::mock_client::MockFollowupTicketClient;

pub(super) const INSTALLATION_ID: i64 = 123;
pub(super) const SENDER_ID: &str = "456";

pub(crate) async fn setup_project(pool: &sqlx::PgPool) -> Uuid {
    test_helpers::ensure_default_team(pool).await;
    let provider_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO integration_providers (id, team_id, name, instance_url, api_key) VALUES ($1, $2, $3, $4, $5)",
        provider_id,
        test_helpers::DEFAULT_TEAM_ID,
        "followup-provider",
        "http://followup-provider.invalid",
        "test-key",
    )
    .execute(pool)
    .await
    .expect("insert provider");
    let project_id = test_helpers::insert_project_config_with_provider(
        pool,
        "github-followup-project",
        provider_id,
    )
    .await;
    sqlx::query!(
        "INSERT INTO project_config_repos (project_config_id, repo_full_name, repo_url, position) VALUES ($1, 'acme/widgets', 'https://github.com/acme/widgets', 0)",
        project_id,
    )
    .execute(pool)
    .await
    .expect("connect repo");
    sqlx::query!(
        "INSERT INTO github_installations (github_installation_id, account_login, team_id) VALUES ($1, 'acme', $2)",
        INSTALLATION_ID,
        test_helpers::DEFAULT_TEAM_ID,
    )
    .execute(pool)
    .await
    .expect("connect installation");
    sqlx::query!(
        "INSERT INTO users (id, email) VALUES ('github-followup-user', 'followup@example.com')",
    )
    .execute(pool)
    .await
    .expect("insert user");
    sqlx::query!(
        "INSERT INTO team_members (team_id, user_id) VALUES ($1, 'github-followup-user')",
        test_helpers::DEFAULT_TEAM_ID,
    )
    .execute(pool)
    .await
    .expect("insert membership");
    sqlx::query!(
        "INSERT INTO user_identities (id, user_id, provider, provider_user_id, provider_login, provider_verified_at) VALUES ($1, 'github-followup-user', 'github', $2, 'octocat', NOW())",
        Uuid::new_v4(),
        SENDER_ID,
    )
    .execute(pool)
    .await
    .expect("insert identity");
    project_id
}

pub(super) async fn service(
    pool: sqlx::PgPool,
    client: Arc<MockFollowupTicketClient>,
) -> WorkRunsService {
    test_helpers::build_state(pool)
        .await
        .jobs
        .clone()
        .with_implementation_followup_ticket_client(client)
}

pub(super) fn request<'a>(
    delivery_id: &'a str,
    request_body: &'a str,
) -> GithubImplementationRequest<'a> {
    GithubImplementationRequest {
        delivery_id,
        installation_id: INSTALLATION_ID,
        comment_id: 789,
        sender_id: SENDER_ID,
        single_user_mode: false,
        repo_full_name: "acme/widgets",
        pr_number: 42,
        pr_title: "Improve retries",
        project_selector: None,
        request_body: Some(request_body),
        command_error: None,
    }
}

pub(super) async fn map_task(pool: &sqlx::PgPool, project_id: Uuid, task_ref: &str) {
    sqlx::query!(
        r#"INSERT INTO task_prs
           (project_config_id, external_task_ref, pr_url, repo_full_name, pr_number)
           VALUES ($1, $2, 'https://github.com/acme/widgets/pull/42', 'acme/widgets', 42)"#,
        project_id,
        task_ref,
    )
    .execute(pool)
    .await
    .expect("map task to PR");
}
