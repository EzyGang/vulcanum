use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use async_trait::async_trait;
use tokio::sync::Notify;
use uuid::Uuid;

use crate::models::project_configs::model::ProjectConfig;
use crate::models::provider_configs::model::IntegrationProvider;
use crate::models::work_runs::errors::WorkRunsError;
use crate::services::work_runs::service::review_ticket::ReviewTicketCreator;
use crate::test_helpers;

pub(super) const INSTALLATION_ID: i64 = 123;
pub(super) const SENDER_ID: &str = "456";

#[derive(Default)]
pub(super) struct MockReviewTicketCreator {
    pub created_count: AtomicUsize,
    pub lookup_count: AtomicUsize,
    existing_task_ref: Mutex<Option<String>>,
    slow_create: bool,
    create_started: Notify,
    create_release: Notify,
}

impl MockReviewTicketCreator {
    pub fn with_existing(external_task_ref: &str) -> Self {
        Self {
            existing_task_ref: Mutex::new(Some(external_task_ref.to_owned())),
            ..Self::default()
        }
    }

    pub fn slow() -> Self {
        Self {
            slow_create: true,
            ..Self::default()
        }
    }

    pub async fn wait_for_create(&self) {
        self.create_started.notified().await;
    }

    pub fn release_create(&self) {
        self.create_release.notify_one();
    }

    pub fn created_count(&self) -> usize {
        self.created_count.load(Ordering::SeqCst)
    }

    pub fn lookup_count(&self) -> usize {
        self.lookup_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl ReviewTicketCreator for MockReviewTicketCreator {
    async fn find_existing(
        &self,
        _provider: &IntegrationProvider,
        _project: &ProjectConfig,
        _repo_full_name: &str,
        _pr_number: i64,
    ) -> Result<Option<String>, WorkRunsError> {
        self.lookup_count.fetch_add(1, Ordering::SeqCst);
        Ok(self
            .existing_task_ref
            .lock()
            .expect("existing task lock")
            .clone())
    }

    async fn create(
        &self,
        _provider: &IntegrationProvider,
        _project: &ProjectConfig,
        repo_full_name: &str,
        pr_number: i64,
        _pr_title: &str,
    ) -> Result<String, WorkRunsError> {
        self.created_count.fetch_add(1, Ordering::SeqCst);
        if self.slow_create {
            self.create_started.notify_one();
            self.create_release.notified().await;
        }
        Ok(format!("review-ticket-{repo_full_name}-{pr_number}"))
    }
}

pub(super) async fn setup_review_project(pool: &sqlx::PgPool) -> Uuid {
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
    connect_repo(pool, project_id).await;
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

pub(super) async fn connect_repo(pool: &sqlx::PgPool, project_id: Uuid) {
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
