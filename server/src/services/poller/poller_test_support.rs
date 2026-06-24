use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::services::model_providers::auth::device_flow::InMemoryDeviceFlowStore;
use crate::services::model_providers::auth::encryption::SecretCipher;
use crate::services::model_providers::auth::openai_chatgpt::OpenAiChatGptDeviceAuthProvider;
use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::repository::ModelProvidersRepository;
use crate::services::model_providers::service::ModelProvidersService;
use crate::services::poller::service::PollerService;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::providers::client::TaskFetcher;
use crate::services::providers::errors::IntegrationError;
use crate::services::providers::model::IntegrationTask;
use crate::services::teams::repository::TeamsRepository;
use crate::services::teams::service::TeamsService;
use crate::services::work_runs::model::{WorkRunStatus, WorkRunType};
use crate::services::work_runs::repository::queries::InsertWorkRunParams;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::test_helpers::DEFAULT_TEAM_ID;

pub(crate) struct MockTaskFetcher {
    responses: RwLock<HashMap<String, Result<Vec<IntegrationTask>, IntegrationError>>>,
}

impl MockTaskFetcher {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            responses: RwLock::new(HashMap::new()),
        }
    }

    pub(crate) async fn set_tasks(
        &self,
        project_id: &str,
        column_slug: &str,
        tasks: Vec<IntegrationTask>,
    ) {
        let key = format!("{}:{}", project_id, column_slug);
        self.responses.write().await.insert(key, Ok(tasks));
    }

    pub(crate) async fn set_error(
        &self,
        project_id: &str,
        column_slug: &str,
        error: IntegrationError,
    ) {
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

#[must_use]
pub(crate) fn make_task(id: &str, title: &str) -> IntegrationTask {
    IntegrationTask {
        id: id.to_owned(),
        title: title.to_owned(),
        project_id: "test-proj".to_owned(),
        description: None,
        number: Some(1),
        project_slug: Some("tst".to_owned()),
    }
}

pub(crate) async fn insert_provider(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();

    crate::test_helpers::ensure_default_team(pool).await;

    sqlx::query!(
        "INSERT INTO integration_providers (id, team_id, name, instance_url, api_key) \
         VALUES ($1, $2, 'Test Provider', 'http://test', 'key')",
        id,
        DEFAULT_TEAM_ID,
    )
    .execute(pool)
    .await
    .expect("Should insert provider");

    id
}

pub(crate) async fn insert_project_config(
    pool: &PgPool,
    external_project_id: &str,
    provider_id: Uuid,
) -> Uuid {
    let id = Uuid::new_v4();

    crate::test_helpers::ensure_default_team(pool).await;

    sqlx::query!(
        "INSERT INTO project_configs \
         (id, team_id, external_project_id, enabled, pickup_column, target_column, progress_column, \
           prompt_template, repo_url, provider_id) \
         VALUES ($1, $2, $3, true, 'to-do', 'in-review', 'in-progress', \
          'Review {{task_title}}', '', $4)",
        id,
        DEFAULT_TEAM_ID,
        external_project_id,
        provider_id,
    )
    .execute(pool)
    .await
    .expect("Should insert project config");

    id
}

pub(crate) async fn insert_active_run(pool: &PgPool, project_config_id: Uuid, task_ref: &str) {
    WorkRunsRepository::new()
        .insert_work_run(
            pool,
            InsertWorkRunParams {
                team_id: DEFAULT_TEAM_ID,
                external_task_ref: task_ref.to_owned(),
                project_config_id,
                prompt_text: "Work".to_owned(),
                repo_url: String::new(),
                repo_full_names: Vec::new(),
                agents_md: String::new(),
                status: WorkRunStatus::Running,
                work_type: WorkRunType::Implementation,
                parent_work_run_id: None,
                task_body: String::new(),
                task_title: Some("Existing work".to_owned()),
                task_slug: None,
                review_target_pr_url: None,
                review_target_repo_full_name: None,
            },
        )
        .await
        .expect("active work run should insert");
}

pub(crate) fn build_service(mock: Arc<MockTaskFetcher>, db: PgPool) -> PollerService {
    let repo = ProjectConfigsRepository::new();
    let model_providers = ModelProvidersService::new(
        ModelProvidersRepository::new(),
        db.clone(),
        ModelCatalogClient::new(),
        SecretCipher::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").expect("test cipher"),
        Arc::new(InMemoryDeviceFlowStore::new()),
        Arc::new(OpenAiChatGptDeviceAuthProvider::new()),
    );
    let project_configs = ProjectConfigsService::new(
        repo.clone(),
        db.clone(),
        crate::services::provider_configs::repository::IntegrationProvidersRepository::new(),
        model_providers,
        TeamsService::new(TeamsRepository::new(), db.clone()),
    );
    let service = PollerService::new(
        project_configs,
        WorkRunsRepository::new(),
        crate::services::provider_configs::repository::IntegrationProvidersRepository::new(),
        db,
        30,
    );

    service.with_fetcher(mock)
}
