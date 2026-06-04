use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::services::integrations::client::TaskFetcher;
use crate::services::integrations::errors::IntegrationError;
use crate::services::integrations::model::IntegrationTask;
use crate::services::poller::service::PollerService;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::work_runs::repository::WorkRunsRepository;

struct MockTaskFetcher {
    responses: RwLock<HashMap<String, Result<Vec<IntegrationTask>, IntegrationError>>>,
}

impl MockTaskFetcher {
    fn new() -> Self {
        Self {
            responses: RwLock::new(HashMap::new()),
        }
    }

    async fn set_tasks(&self, project_id: &str, column_slug: &str, tasks: Vec<IntegrationTask>) {
        let key = format!("{}:{}", project_id, column_slug);
        self.responses.write().await.insert(key, Ok(tasks));
    }

    async fn set_error(&self, project_id: &str, column_slug: &str, error: IntegrationError) {
        let key = format!("{}:{}", project_id, column_slug);
        self.responses.write().await.insert(key, Err(error));
    }
}

#[async_trait]
impl TaskFetcher for MockTaskFetcher {
    async fn fetch_tasks_in_column(
        &self,
        project_id: &str,
        column_slug: &str,
    ) -> Result<Vec<IntegrationTask>, IntegrationError> {
        let key = format!("{}:{}", project_id, column_slug);
        match self.responses.read().await.get(&key) {
            Some(Ok(tasks)) => Ok(tasks.clone()),
            Some(Err(e)) => Err(IntegrationError::Other(format!("{}", e))),
            None => Err(IntegrationError::Other("unreachable".to_owned())),
        }
    }
}

fn make_task(id: &str, title: &str) -> IntegrationTask {
    IntegrationTask {
        id: id.to_owned(),
        title: title.to_owned(),
        project_id: "test-proj".to_owned(),
        description: None,
    }
}

async fn insert_provider(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO integration_providers (id, name, instance_url, api_key) \
         VALUES ($1, 'Test Provider', 'http://test', 'key')",
        id,
    )
    .execute(pool)
    .await
    .expect("Should insert provider");

    id
}

async fn insert_project_config(pool: &PgPool, kaneo_project_id: &str, provider_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs \
         (id, kaneo_project_id, enabled, pickup_column, target_column, progress_column, \
          prompt_template, repo_url, provider_id) \
         VALUES ($1, $2, true, 'to-do', 'in-review', 'in-progress', \
          'Review {{task_title}}', '', $3)",
        id,
        kaneo_project_id,
        provider_id,
    )
    .execute(pool)
    .await
    .expect("Should insert project config");

    id
}

fn build_service(mock: Arc<MockTaskFetcher>, db: PgPool) -> PollerService {
    let repo = ProjectConfigsRepository::new();
    let service = PollerService::new(
        repo.clone(),
        WorkRunsRepository::new(),
        crate::services::integration_providers::repository::IntegrationProvidersRepository::new(),
        db,
        30,
    );

    service.with_fetcher(mock)
}

#[sqlx::test]
async fn poller_inserts_tasks(pool: PgPool) {
    let mock = Arc::new(MockTaskFetcher::new());
    let provider_id = insert_provider(&pool).await;
    let project_id = insert_project_config(&pool, "kaneo-proj-1", provider_id).await;

    mock.set_tasks(
        "kaneo-proj-1",
        "to-do",
        vec![
            make_task("task-1", "Fix login bug"),
            make_task("task-2", "Add dark mode"),
        ],
    )
    .await;

    let service = build_service(mock, pool.clone());
    service.poll_once().await;

    let rows = sqlx::query!(
        "SELECT external_task_ref, prompt_text FROM work_runs \
         WHERE project_config_id = $1 ORDER BY external_task_ref",
        project_id,
    )
    .fetch_all(&pool)
    .await
    .expect("Should query work_runs");

    assert_eq!(rows.len(), 2, "Should insert 2 work_runs");
    assert_eq!(rows[0].external_task_ref, "task-1");
    assert!(rows[0].prompt_text.starts_with("Review Fix login bug"));
    assert!(rows[0].prompt_text.contains("Debian-based container"));
    assert_eq!(rows[1].external_task_ref, "task-2");
    assert!(rows[1].prompt_text.starts_with("Review Add dark mode"));
    assert!(rows[1].prompt_text.contains("Debian-based container"));
}

#[sqlx::test]
async fn poller_skips_duplicates(pool: PgPool) {
    let mock = Arc::new(MockTaskFetcher::new());
    let provider_id = insert_provider(&pool).await;

    let _project_id = insert_project_config(&pool, "kaneo-proj-2", provider_id).await;

    mock.set_tasks(
        "kaneo-proj-2",
        "to-do",
        vec![make_task("task-dup", "Fix login bug")],
    )
    .await;

    let service = build_service(mock.clone(), pool.clone());

    service.poll_once().await;
    service.poll_once().await;

    let row = sqlx::query!(
        "SELECT COUNT(*) as count FROM work_runs WHERE external_task_ref = $1",
        "task-dup",
    )
    .fetch_one(&pool)
    .await
    .expect("Should query work_runs");

    assert_eq!(
        row.count.unwrap(),
        1,
        "Should not insert duplicate work_run"
    );
}

#[sqlx::test]
async fn poller_handles_unreachable_kaneo(pool: PgPool) {
    let mock = Arc::new(MockTaskFetcher::new());
    let provider_id = insert_provider(&pool).await;

    insert_project_config(&pool, "kaneo-good", provider_id).await;
    insert_project_config(&pool, "kaneo-bad", provider_id).await;

    mock.set_tasks(
        "kaneo-good",
        "to-do",
        vec![make_task("task-ok", "Working project task")],
    )
    .await;
    mock.set_error(
        "kaneo-bad",
        "to-do",
        IntegrationError::Other("connection refused".to_owned()),
    )
    .await;

    let service = build_service(mock, pool.clone());
    service.poll_once().await;

    let row = sqlx::query!(
        "SELECT COUNT(*) as count FROM work_runs WHERE external_task_ref = $1",
        "task-ok",
    )
    .fetch_one(&pool)
    .await
    .expect("Should query work_runs");

    assert_eq!(
        row.count.unwrap(),
        1,
        "Should insert task from working project despite failing one"
    );
}
