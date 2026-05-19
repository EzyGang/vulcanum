use std::sync::Arc;

use sqlx::PgPool;

use crate::config::AppConfig;
use crate::services::auth::service::AuthService;
use crate::services::kaneo::client::KaneoClient;
use crate::services::poller::notifier::WorkNotifier;
use crate::services::poller::service::PollerService;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::users::repository::UsersRepository;
use crate::services::users::service::UsersService;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::services::workers::repository::WorkersRepository;
use crate::services::workers::service::WorkersService;

#[derive(Clone)]
pub struct AppState {
    pub auth: AuthService,
    pub project_configs: ProjectConfigsService,
    pub workers: WorkersService,
    pub db_pool: PgPool,
    pub kaneo: KaneoClient,
    pub work_runs: WorkRunsRepository,
    pub work_notifier: WorkNotifier,
}

impl AppState {
    pub async fn new(cfg: &AppConfig) -> Result<Self, sqlx::Error> {
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(cfg.max_conns)
            .connect(&cfg.db_url)
            .await?;

        let kaneo = KaneoClient::new(
            std::env::var("KANEO_INSTANCE").unwrap_or_else(|_| "cloud.kaneo.app".to_owned()),
            std::env::var("KANEO_API_KEY").unwrap_or_default(),
        );

        let users = UsersService::new(UsersRepository::new(), db_pool.clone());
        let auth = AuthService::new(users);
        let project_configs_repo = ProjectConfigsRepository::new();
        let project_configs = ProjectConfigsService::new(
            project_configs_repo.clone(),
            db_pool.clone(),
            kaneo.clone(),
        );
        let workers_repo = WorkersRepository::new();
        let workers = WorkersService::new(workers_repo, db_pool.clone(), cfg);
        let work_runs = WorkRunsRepository::new();
        let work_notifier = WorkNotifier::new();

        Ok(Self {
            auth,
            project_configs,
            workers,
            db_pool,
            kaneo,
            work_runs,
            work_notifier,
        })
    }

    pub fn into_poller(self, poll_period_secs: u64) -> PollerService {
        PollerService::new(
            Arc::new(self.kaneo.clone()),
            self.project_configs.repo.clone(),
            self.work_runs.clone(),
            self.db_pool.clone(),
            poll_period_secs,
            self.work_notifier.clone(),
        )
    }
}
