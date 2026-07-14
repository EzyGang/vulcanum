use std::sync::Arc;

use crate::config::AppConfig;
use crate::db::auth::AuthRepository;
use crate::db::github_app::GithubAppRepository;
use crate::db::model_providers::ModelProvidersRepository;
use crate::db::project_configs::ProjectConfigsRepository;
use crate::db::provider_configs::IntegrationProvidersRepository;
use crate::db::task_augmentations::TaskAugmentationsRepository;
use crate::db::teams::TeamsRepository;
use crate::db::users::UsersRepository;
use crate::db::work_run_events::WorkRunEventsRepository;
use crate::db::work_runs::WorkRunsRepository;
use crate::db::workers::WorkersRepository;
use crate::services::auth::service::AuthService;
use crate::services::dispatcher::cancel_store::{
    CancelStore, InMemoryCancelStore, RedisCancelStore,
};
use crate::services::dispatcher::dispatch_store::DispatchStore;
use crate::services::github_app::service::webhooks::GithubWebhookService;
use crate::services::github_app::service::GithubAppManager;
use crate::services::model_providers::auth::device_flow::RedisDeviceFlowStore;
use crate::services::model_providers::auth::encryption::SecretCipher;
use crate::services::model_providers::auth::openai_chatgpt::OpenAiChatGptDeviceAuthProvider;
use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::service::ModelProvidersService;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::provider_configs::service::IntegrationProvidersService;
use crate::services::task_board::service::TaskBoardService;
use crate::services::teams::invite_store::RedisTeamInviteStore;
use crate::services::teams::service::TeamsService;
use crate::services::users::service::UsersService;
use crate::services::work_run_events::service::WorkRunEventsService;
use crate::services::work_runs::service::WorkRunsService;
use crate::services::workers::registration_code_store::RedisCodeStore;
use crate::services::workers::service::WorkersService;

#[derive(Clone)]
pub struct AppState {
    pub auth: AuthService,
    pub project_configs: ProjectConfigsService,
    pub providers: IntegrationProvidersService,
    pub task_board: TaskBoardService,
    pub model_providers: ModelProvidersService,
    pub workers: WorkersService,
    pub jobs: WorkRunsService,
    pub events: WorkRunEventsService,
    pub github: GithubAppManager,
    pub github_webhooks: GithubWebhookService,
    pub teams: TeamsService,
    pub jwt_secret: String,
    pub is_single_user: bool,
}

impl AppState {
    pub async fn new(cfg: &AppConfig) -> Result<Self, eyre::Error> {
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(cfg.max_conns)
            .connect(&cfg.db_url)
            .await?;

        let providers_repo = IntegrationProvidersRepository::new();
        let providers = IntegrationProvidersService::new(providers_repo.clone(), db_pool.clone());
        let model_catalog = ModelCatalogClient::new()?;
        let model_providers_repo = ModelProvidersRepository::new();
        let model_provider_cipher = SecretCipher::new(&cfg.model_provider_secret_key)?;
        let device_flow_store = Arc::new(RedisDeviceFlowStore::new(&cfg.redis_url)?);
        let device_auth_provider = Arc::new(OpenAiChatGptDeviceAuthProvider::new()?);
        let model_providers = ModelProvidersService::new(
            model_providers_repo.clone(),
            db_pool.clone(),
            model_catalog.clone(),
            model_provider_cipher,
            device_flow_store,
            device_auth_provider,
        );
        let invite_store = RedisTeamInviteStore::new(&cfg.redis_url)?;
        let teams = TeamsService::new_with_invite_store(
            TeamsRepository::new(),
            db_pool.clone(),
            Arc::new(invite_store),
        );

        let users = UsersService::new(UsersRepository::new(), db_pool.clone());
        let auth = AuthService::new(
            AuthRepository::new(),
            db_pool.clone(),
            users,
            teams.clone(),
            cfg.instance_password.clone(),
            cfg.jwt_secret.clone(),
            cfg,
        )?;
        let project_configs_repo = ProjectConfigsRepository::new();
        let task_board = TaskBoardService::new(
            db_pool.clone(),
            providers_repo.clone(),
            project_configs_repo.clone(),
            TaskAugmentationsRepository::new(),
        );
        let project_configs = ProjectConfigsService::new(
            project_configs_repo.clone(),
            db_pool.clone(),
            providers_repo.clone(),
            teams.clone(),
        );
        let workers_repo = WorkersRepository::new();
        let code_store = RedisCodeStore::new(&cfg.redis_url)?;
        let work_runs_repo_for_workers = WorkRunsRepository::new();
        let workers = WorkersService::new(
            workers_repo.clone(),
            work_runs_repo_for_workers,
            db_pool.clone(),
            cfg,
            Arc::new(code_store),
        );
        let github = GithubAppManager::new(
            GithubAppRepository::new(),
            db_pool.clone(),
            &cfg.redis_url,
            cfg,
        )?;
        let work_runs = WorkRunsRepository::new();
        let dispatch_store: Arc<dyn DispatchStore> = Arc::new(
            crate::services::dispatcher::dispatch_store::RedisDispatchStore::new(&cfg.redis_url)?,
        );
        let cancel_store: Arc<dyn CancelStore> = Arc::new(RedisCancelStore::new(&cfg.redis_url)?);
        let jobs = WorkRunsService::new(
            work_runs.clone(),
            TaskAugmentationsRepository::new(),
            workers_repo,
            project_configs.clone(),
            github.clone(),
            db_pool.clone(),
            dispatch_store.clone(),
            providers_repo.clone(),
            model_providers.clone(),
            cancel_store.clone(),
            cfg.unhealthy_threshold,
        );
        let github_webhooks =
            GithubWebhookService::new(cfg.github_webhook_secret.as_deref(), jobs.clone());
        let events = WorkRunEventsService::new(
            WorkRunEventsRepository::new(),
            work_runs.clone(),
            cancel_store.clone(),
            db_pool.clone(),
        );

        let jwt_secret = cfg.jwt_secret.clone();

        Ok(Self {
            auth,
            project_configs,
            providers,
            task_board,
            model_providers,
            workers,
            jobs,
            events,
            github,
            github_webhooks,
            teams,
            jwt_secret,
            is_single_user: cfg.is_single_user,
        })
    }

    pub async fn run_migrations(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!().run(&self.jobs.db).await
    }

    pub fn into_poller(
        self,
        poll_period_secs: u64,
    ) -> crate::services::poller::service::PollerService {
        let jobs = self.jobs.clone();
        crate::services::poller::service::PollerService::new(
            self.project_configs.clone(),
            jobs.work_runs_repo.clone(),
            self.providers.repository(),
            jobs.db.clone(),
            poll_period_secs,
        )
    }
}

/// Build an `AppState` with an in-memory cancel store. Used in tests
/// and any path that does not have a real Redis instance available.
#[allow(dead_code)]
pub fn build_in_memory_cancel_store() -> Arc<dyn CancelStore> {
    Arc::new(InMemoryCancelStore::new())
}
