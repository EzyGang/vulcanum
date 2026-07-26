mod job_access;
mod polling_and_ack;
mod results;

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;
use vulcanum_shared::api::wire::AgentBackend;

use crate::db::github_app::GithubAppRepository;
use crate::db::model_providers::ModelProvidersRepository;
use crate::db::project_configs::ProjectConfigsRepository;
use crate::db::project_usage::ProjectUsageRepository;
use crate::db::provider_configs::IntegrationProvidersRepository;
use crate::db::task_augmentations::TaskAugmentationsRepository;
use crate::db::teams::TeamsRepository;
use crate::db::work_runs::WorkRunsRepository;
use crate::db::workers::WorkersRepository;
use crate::models::providers::errors::IntegrationError;
use crate::models::providers::model::IntegrationTask;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::WorkRunStatus;
use crate::models::workers::model::WorkerStatus;
use crate::services::dispatcher::cancel_store::InMemoryCancelStore;
use crate::services::dispatcher::dispatch_store::InMemoryDispatchStore;
use crate::services::github_app::service::GithubAppManager;
use crate::services::model_providers::auth::device_flow::InMemoryDeviceFlowStore;
use crate::services::model_providers::auth::encryption::SecretCipher;
use crate::services::model_providers::auth::openai_chatgpt::OpenAiChatGptDeviceAuthProvider;
use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::service::ModelProvidersService;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::providers::client::TaskFetcher;
use crate::services::teams::service::TeamsService;
use crate::services::work_runs::service::WorkRunsService;
use crate::test_helpers;
use vulcanum_shared::api::wire::SubmitResultRequest;

fn build_github_manager(pool: sqlx::PgPool) -> GithubAppManager {
    let cfg = crate::config::AppConfig {
        db_url: String::new(),
        max_conns: 1,
        poll_period_secs: 30,
        jwt_secret: String::new(),
        stale_worker_threshold_secs: 120,
        unhealthy_threshold: 3,
        stalled_running_threshold_secs: 1800,
        instance_password: String::new(),
        is_single_user: true,
        redis_url: "redis://127.0.0.1:6379".to_owned(),
        model_provider_secret_key: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_owned(),
        github_app_id: None,
        github_app_private_key: None,
        github_app_slug: None,
        github_webhook_secret: None,
        github_oauth_client_id: None,
        github_oauth_client_secret: None,
        github_oauth_redirect_url: None,
    };
    GithubAppManager::new(
        GithubAppRepository::new(),
        pool,
        "redis://127.0.0.1:6379",
        &cfg,
    )
    .expect("build github manager for tests")
}

fn build_service(pool: sqlx::PgPool) -> WorkRunsService {
    let model_catalog = ModelCatalogClient::new().expect("build model catalog client");
    let model_providers_repo = ModelProvidersRepository::new();
    let model_providers = ModelProvidersService::new(
        model_providers_repo.clone(),
        pool.clone(),
        model_catalog,
        SecretCipher::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").expect("test cipher"),
        Arc::new(InMemoryDeviceFlowStore::new()),
        Arc::new(OpenAiChatGptDeviceAuthProvider::new().expect("build device auth client")),
    );
    let project_configs = ProjectConfigsService::new(
        ProjectConfigsRepository::new(),
        pool.clone(),
        IntegrationProvidersRepository::new(),
        TeamsService::new(TeamsRepository::new(), pool.clone()),
    );
    WorkRunsService::new(
        WorkRunsRepository::new(),
        TaskAugmentationsRepository::new(),
        ProjectUsageRepository::new(),
        WorkersRepository::new(),
        project_configs,
        build_github_manager(pool.clone()),
        pool,
        Arc::new(InMemoryDispatchStore::default()),
        IntegrationProvidersRepository::new(),
        model_providers,
        Arc::new(InMemoryCancelStore::new()),
        3,
    )
}

struct StaticTaskFetcher {
    task: IntegrationTask,
}

#[async_trait]
impl TaskFetcher for StaticTaskFetcher {
    async fn fetch_tasks_in_column(
        &self,
        _project_id: &str,
        _column_name: &str,
    ) -> Result<Vec<IntegrationTask>, IntegrationError> {
        Ok(vec![self.task.clone()])
    }

    async fn fetch_task(&self, _task_id: &str) -> Result<IntegrationTask, IntegrationError> {
        Ok(self.task.clone())
    }

    async fn update_task_status(
        &self,
        _task_id: &str,
        _new_status: &str,
    ) -> Result<(), IntegrationError> {
        Ok(())
    }
}

fn static_task(title: &str, description: &str) -> Arc<StaticTaskFetcher> {
    Arc::new(StaticTaskFetcher {
        task: IntegrationTask {
            id: "task-get".to_owned(),
            title: title.to_owned(),
            project_id: "kaneo-get-1".to_owned(),
            description: Some(description.to_owned()),
            status: "in-progress".to_owned(),
            priority: "low".to_owned(),
            number: None,
            project_slug: None,
            assignee_name: None,
            created_at: "2026-01-01T00:00:00Z".to_owned(),
            updated_at: None,
            labels: Vec::new(),
        },
    })
}

fn completed_result_request() -> SubmitResultRequest {
    SubmitResultRequest {
        pr_urls: Vec::new(),
        exit_code: 0,
        tokens_used: 0,
        duration_ms: 1000,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: None,
        result_summary: None,
        review_url: None,
        review_body: None,
        review_already_exists: false,
    }
}

async fn zero_worker_active_jobs(pool: &sqlx::PgPool, worker_id: Uuid) {
    sqlx::query!(
        "UPDATE workers SET active_jobs = 0 WHERE id = $1",
        worker_id
    )
    .execute(pool)
    .await
    .expect("Should zero active jobs");
}
