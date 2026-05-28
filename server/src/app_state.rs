use std::sync::Arc;

use sqlx::PgPool;

use crate::config::AppConfig;
use crate::services::auth::service::AuthService;
use crate::services::dispatcher::flag_store::DispatchStore;
use crate::services::integrations::client::IntegrationClient;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::users::repository::UsersRepository;
use crate::services::users::service::UsersService;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::services::work_runs::service::WorkRunsService;
use crate::services::workers::code_store::RedisCodeStore;
use crate::services::workers::repository::WorkersRepository;
use crate::services::workers::service::WorkersService;

#[derive(Clone)]
pub struct AppState {
    pub auth: AuthService,
    pub project_configs: ProjectConfigsService,
    pub workers: WorkersService,
    pub jobs: WorkRunsService,
    pub db_pool: PgPool,
    pub integration: IntegrationClient,
    pub work_runs: WorkRunsRepository,
    pub dispatch_store: Arc<dyn DispatchStore>,
    pub jwt_secret: String,
}

impl AppState {
    pub async fn new(cfg: &AppConfig) -> Result<Self, eyre::Error> {
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(cfg.max_conns)
            .connect(&cfg.db_url)
            .await?;

        let kaneo =
            IntegrationClient::new_kaneo(cfg.kaneo_instance.clone(), cfg.kaneo_api_key.clone());

        let users = UsersService::new(UsersRepository::new(), db_pool.clone());
        let auth = AuthService::new(users, cfg.instance_password.clone(), cfg.jwt_secret.clone());
        let project_configs_repo = ProjectConfigsRepository::new();
        let project_configs = ProjectConfigsService::new(
            project_configs_repo.clone(),
            db_pool.clone(),
            kaneo.clone(),
        );
        let workers_repo = WorkersRepository::new();
        let code_store = RedisCodeStore::new(&cfg.redis_url)?;
        let workers = WorkersService::new(
            workers_repo.clone(),
            db_pool.clone(),
            cfg,
            Arc::new(code_store),
        );
        let work_runs = WorkRunsRepository::new();
        let dispatch_store: Arc<dyn DispatchStore> = Arc::new(
            crate::services::dispatcher::flag_store::RedisDispatchStore::new(&cfg.redis_url)?,
        );
        let jobs = WorkRunsService::new(
            work_runs.clone(),
            workers_repo,
            project_configs_repo,
            db_pool.clone(),
            dispatch_store.clone(),
            kaneo.clone(),
        );

        let jwt_secret = cfg.jwt_secret.clone();

        Ok(Self {
            auth,
            project_configs,
            workers,
            jobs,
            db_pool,
            integration: kaneo,
            work_runs,
            dispatch_store,
            jwt_secret,
        })
    }

    pub fn into_poller(
        self,
        poll_period_secs: u64,
    ) -> crate::services::poller::service::PollerService {
        crate::services::poller::service::PollerService::new(
            Arc::new(self.integration.clone()),
            self.project_configs.repo.clone(),
            self.work_runs.clone(),
            self.db_pool.clone(),
            poll_period_secs,
        )
    }
}
