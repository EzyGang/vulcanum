use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::github_app::errors::GithubAppError;
use crate::models::providers::errors::IntegrationError;
use crate::models::providers::model::IntegrationTask;
use crate::services::github_app::service::pull_requests::{
    PullRequestState, PullRequestStateReader,
};
use crate::services::github_app::service::webhooks::tests::APP_SLUG;
use crate::services::github_app::service::webhooks::{GithubWebhookOutcome, GithubWebhookService};
use crate::services::github_app::webhook_store::GithubWebhookStore;
use crate::services::providers::client::TaskFetcher;
use crate::test_helpers;

struct MergedPullRequestReader;

#[async_trait]
impl PullRequestStateReader for MergedPullRequestReader {
    async fn pull_request_state(
        &self,
        _team_id: Uuid,
        _repo_full_name: &str,
        _number: i64,
    ) -> Result<PullRequestState, GithubAppError> {
        Ok(PullRequestState::Merged)
    }
}

struct RecordingTaskFetcher {
    status: Mutex<String>,
    updates: Mutex<Vec<(String, String)>>,
}

#[async_trait]
impl TaskFetcher for RecordingTaskFetcher {
    async fn fetch_tasks_in_column(
        &self,
        _project_id: &str,
        _column_name: &str,
    ) -> Result<Vec<IntegrationTask>, IntegrationError> {
        Err(IntegrationError::Other(
            "unexpected task list request".to_owned(),
        ))
    }

    async fn fetch_task(&self, task_id: &str) -> Result<IntegrationTask, IntegrationError> {
        Ok(IntegrationTask {
            id: task_id.to_owned(),
            title: "Merged pull request".to_owned(),
            project_id: "project-1".to_owned(),
            description: None,
            status: self.status.lock().await.clone(),
            priority: "medium".to_owned(),
            number: Some(1),
            project_slug: Some("project".to_owned()),
            assignee_name: None,
            created_at: "2026-07-27T00:00:00Z".to_owned(),
            updated_at: None,
            labels: Vec::new(),
        })
    }

    async fn update_task_status(
        &self,
        task_id: &str,
        new_status: &str,
    ) -> Result<(), IntegrationError> {
        *self.status.lock().await = new_status.to_owned();
        self.updates
            .lock()
            .await
            .push((task_id.to_owned(), new_status.to_owned()));
        Ok(())
    }
}

#[sqlx::test]
async fn merged_webhook_matches_pr_url_without_installation_mapping(pool: sqlx::PgPool) {
    let (service, task_fetcher) = completion_service(&pool).await;
    let payload = test_helpers::github::github_pull_request_payload("closed", true);
    let signature = test_helpers::github::sign_github_webhook(&payload);

    assert_eq!(
        service
            .handle(&signature, "pull_request", "merged-delivery", &payload)
            .await
            .expect("queue merged delivery"),
        GithubWebhookOutcome::Queued { inserted: true },
    );
    assert!(service
        .process_pending_once()
        .await
        .expect("process merged delivery"));

    assert_eq!(task_fetcher.status.lock().await.as_str(), "done");
    assert_eq!(
        task_fetcher.updates.lock().await.as_slice(),
        &[("task-1".to_owned(), "done".to_owned())]
    );
}

#[sqlx::test]
async fn closed_unmerged_webhook_does_not_queue_or_move_mapped_ticket(pool: sqlx::PgPool) {
    let (service, task_fetcher) = completion_service(&pool).await;
    let payload = test_helpers::github::github_pull_request_payload("closed", false);
    let signature = test_helpers::github::sign_github_webhook(&payload);

    assert_eq!(
        service
            .handle(&signature, "pull_request", "closed-delivery", &payload)
            .await
            .expect("ignore unmerged delivery"),
        GithubWebhookOutcome::Ignored,
    );
    assert!(!service
        .process_pending_once()
        .await
        .expect("check completion queue"));
    assert_eq!(task_fetcher.status.lock().await.as_str(), "in review");
    assert!(task_fetcher.updates.lock().await.is_empty());
}

async fn completion_service(
    pool: &sqlx::PgPool,
) -> (GithubWebhookService, Arc<RecordingTaskFetcher>) {
    test_helpers::teams::ensure_default_team(pool).await;
    let provider_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO integration_providers \
         (id, team_id, name, instance_url, api_key) VALUES ($1, $2, $3, $4, $5)",
        provider_id,
        test_helpers::DEFAULT_TEAM_ID,
        "completion-provider",
        "http://127.0.0.1:1",
        "test-key",
    )
    .execute(pool)
    .await
    .expect("insert completion provider");
    let project_config_id = test_helpers::project_configs::insert_project_config_with_provider(
        pool,
        "project-1",
        provider_id,
    )
    .await;
    sqlx::query!(
        "INSERT INTO task_prs \
         (project_config_id, external_task_ref, pr_url, repo_full_name, pr_number) \
         VALUES ($1, $2, $3, $4, $5)",
        project_config_id,
        "task-1",
        "https://github.com/acme/widgets/pull/42",
        "acme/widgets",
        42_i64,
    )
    .execute(pool)
    .await
    .expect("insert task PR mapping");

    let state = test_helpers::state::build_state(pool.clone()).await;
    let task_fetcher = Arc::new(RecordingTaskFetcher {
        status: Mutex::new("in review".to_owned()),
        updates: Mutex::new(Vec::new()),
    });
    let work_runs = state
        .jobs
        .clone()
        .with_pr_state_reader(Arc::new(MergedPullRequestReader))
        .with_task_fetcher(task_fetcher.clone());
    let service = GithubWebhookService::new(
        Some(Arc::from(test_helpers::GITHUB_WEBHOOK_SECRET)),
        Some(Arc::from(APP_SLUG)),
        state.is_single_user,
        GithubWebhookStore::in_memory(),
        work_runs,
        Arc::new(state.github.clone()),
    );

    (service, task_fetcher)
}
