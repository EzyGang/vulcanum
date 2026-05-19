use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use kaneo_cli::api::types::Task;
use sqlx::PgPool;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::services::kaneo::client::TaskFetcher;
use crate::services::kaneo::errors::KaneoError;
use crate::services::poller::notifier::WorkNotifier;
use crate::services::poller::service::PollerService;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::work_runs::repository::WorkRunsRepository;

struct MockTaskFetcher {
    responses: RwLock<HashMap<String, Result<Vec<Task>, KaneoError>>>,
}

impl MockTaskFetcher {
    fn new() -> Self {
        Self {
            responses: RwLock::new(HashMap::new()),
        }
    }

    async fn set_tasks(&self, project_id: &str, column_slug: &str, tasks: Vec<Task>) {
        let key = format!("{}:{}", project_id, column_slug);
        self.responses.write().await.insert(key, Ok(tasks));
    }

    async fn set_error(&self, project_id: &str, column_slug: &str, error: KaneoError) {
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
    ) -> Result<Vec<Task>, KaneoError> {
        let key = format!("{}:{}", project_id, column_slug);
        match self.responses.read().await.get(&key) {
            Some(result) => match result {
                Ok(tasks) => Ok(tasks.clone()),
                Err(e) => Err(KaneoError::Api(format!("{}", e))),
            },
            None => Err(KaneoError::Api("unreachable".to_owned())),
        }
    }
}

fn make_task(id: &str, title: &str) -> Task {
    Task {
        id: id.to_owned(),
        project_id: "test-proj".to_owned(),
        position: None,
        number: None,
        user_id: None,
        title: title.to_owned(),
        description: None,
        status: "to-do".to_owned(),
        priority: "medium".to_owned(),
        due_date: None,
        created_at: "2026-01-01T00:00:00Z".to_owned(),
        start_date: None,
        updated_at: None,
        column_id: None,
        assignee_name: None,
        assignee_id: None,
        assignee_image: None,
    }
}

async fn insert_project_config(pool: &PgPool, kaneo_project_id: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs (id, kaneo_project_id, enabled, pickup_column, target_column, progress_column, prompt_template, repo_url) \
         VALUES ($1, $2, true, 'to-do', 'in-review', 'in-progress', 'Review {{task_title}}', '')",
        id,
        kaneo_project_id,
    )
    .execute(pool)
    .await
    .expect("Should insert project config");

    id
}

fn build_service(mock: Arc<MockTaskFetcher>, db: PgPool, notifier: WorkNotifier) -> PollerService {
    let kaneo: Arc<dyn TaskFetcher> = mock;

    PollerService::new(
        kaneo,
        ProjectConfigsRepository::new(),
        WorkRunsRepository::new(),
        db,
        30,
        notifier,
    )
}

#[sqlx::test]
async fn poller_inserts_tasks(pool: PgPool) {
    let mock = Arc::new(MockTaskFetcher::new());
    let notifier = WorkNotifier::new();

    let project_id = insert_project_config(&pool, "kaneo-proj-1").await;

    mock.set_tasks(
        "kaneo-proj-1",
        "to-do",
        vec![
            make_task("task-1", "Fix login bug"),
            make_task("task-2", "Add dark mode"),
        ],
    )
    .await;

    let service = build_service(mock, pool.clone(), notifier);
    service.poll_once().await;

    let row = sqlx::query!(
        "SELECT COUNT(*) as count FROM work_runs WHERE project_config_id = $1",
        project_id,
    )
    .fetch_one(&pool)
    .await
    .expect("Should query work_runs");

    assert_eq!(row.count.unwrap(), 2, "Should insert 2 work_runs");
}

#[sqlx::test]
async fn poller_skips_duplicates(pool: PgPool) {
    let mock = Arc::new(MockTaskFetcher::new());
    let notifier = WorkNotifier::new();

    let _project_id = insert_project_config(&pool, "kaneo-proj-2").await;

    mock.set_tasks(
        "kaneo-proj-2",
        "to-do",
        vec![make_task("task-dup", "Fix login bug")],
    )
    .await;

    let service = build_service(mock.clone(), pool.clone(), notifier);

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
    let notifier = WorkNotifier::new();

    insert_project_config(&pool, "kaneo-good").await;
    insert_project_config(&pool, "kaneo-bad").await;

    mock.set_tasks(
        "kaneo-good",
        "to-do",
        vec![make_task("task-ok", "Working project task")],
    )
    .await;
    mock.set_error(
        "kaneo-bad",
        "to-do",
        KaneoError::Api("connection refused".to_owned()),
    )
    .await;

    let service = build_service(mock, pool.clone(), notifier);
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

#[sqlx::test]
async fn poller_flips_notifier(pool: PgPool) {
    let mock = Arc::new(MockTaskFetcher::new());
    let notifier = WorkNotifier::new();

    let worker_id = Uuid::new_v4();
    notifier.add_worker(worker_id).await;

    insert_project_config(&pool, "kaneo-notify").await;

    mock.set_tasks(
        "kaneo-notify",
        "to-do",
        vec![make_task("task-new", "New task")],
    )
    .await;

    let service = build_service(mock, pool.clone(), notifier.clone());
    service.poll_once().await;

    let has_work = notifier.take(&worker_id).await;
    assert!(has_work, "Notifier should flag new work");
}
