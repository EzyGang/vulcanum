use std::sync::Arc;

use sqlx::PgPool;

use crate::config::AppConfig;
use crate::services::auth::service::AuthService;
use crate::services::dispatcher::cancel_store::{
    CancelStore, InMemoryCancelStore, RedisCancelStore,
};
use crate::services::dispatcher::flag_store::DispatchStore;
use crate::services::integration_providers::repository::IntegrationProvidersRepository;
use crate::services::integration_providers::service::IntegrationProvidersService;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::users::repository::UsersRepository;
use crate::services::users::service::UsersService;
use crate::services::work_run_events::repository::WorkRunEventsRepository;
use crate::services::work_run_events::service::WorkRunEventsService;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::services::work_runs::service::WorkRunsService;
use crate::services::workers::code_store::RedisCodeStore;
use crate::services::workers::repository::WorkersRepository;
use crate::services::workers::service::WorkersService;

#[derive(Clone)]
pub struct AppState {
    pub auth: AuthService,
    pub project_configs: ProjectConfigsService,
    pub providers: IntegrationProvidersService,
    pub workers: WorkersService,
    pub jobs: WorkRunsService,
    pub events: WorkRunEventsService,
    pub db_pool: PgPool,
    pub work_runs: WorkRunsRepository,
    pub dispatch_store: Arc<dyn DispatchStore>,
    pub cancel_store: Arc<dyn CancelStore>,
    pub jwt_secret: String,
}

impl AppState {
    pub async fn new(cfg: &AppConfig) -> Result<Self, eyre::Error> {
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(cfg.max_conns)
            .connect(&cfg.db_url)
            .await?;

        let providers_repo = IntegrationProvidersRepository::new();
        let providers = IntegrationProvidersService::new(providers_repo.clone(), db_pool.clone());

        let users = UsersService::new(UsersRepository::new(), db_pool.clone());
        let auth = AuthService::new(users, cfg.instance_password.clone(), cfg.jwt_secret.clone());
        let project_configs_repo = ProjectConfigsRepository::new();
        let project_configs = ProjectConfigsService::new(
            project_configs_repo.clone(),
            db_pool.clone(),
            providers_repo.clone(),
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
        let work_runs = WorkRunsRepository::new();
        let dispatch_store: Arc<dyn DispatchStore> = Arc::new(
            crate::services::dispatcher::flag_store::RedisDispatchStore::new(&cfg.redis_url)?,
        );
        let cancel_store: Arc<dyn CancelStore> = Arc::new(RedisCancelStore::new(&cfg.redis_url)?);
        let jobs = WorkRunsService::new(
            work_runs.clone(),
            workers_repo,
            project_configs_repo,
            db_pool.clone(),
            dispatch_store.clone(),
            providers_repo.clone(),
            cancel_store.clone(),
            cfg.unhealthy_threshold,
        );
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
            workers,
            jobs,
            events,
            db_pool,
            work_runs,
            dispatch_store,
            cancel_store,
            jwt_secret,
        })
    }

    pub fn into_poller(
        self,
        poll_period_secs: u64,
    ) -> crate::services::poller::service::PollerService {
        let providers_repo = self.providers.repo.clone();
        crate::services::poller::service::PollerService::new(
            self.project_configs.repo.clone(),
            self.work_runs.clone(),
            providers_repo,
            self.db_pool.clone(),
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
