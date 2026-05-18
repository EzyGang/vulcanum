use sqlx::PgPool;

use crate::config::AppConfig;
use crate::services::auth::service::AuthService;
use crate::services::kaneo::client::KaneoClient;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::users::repository::UsersRepository;
use crate::services::users::service::UsersService;

#[derive(Clone)]
pub struct AppState {
    pub auth: AuthService,
    pub project_configs: ProjectConfigsService,
    pub db_pool: PgPool,
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
        let project_configs =
            ProjectConfigsService::new(ProjectConfigsRepository::new(), db_pool.clone(), kaneo);

        Ok(Self {
            auth,
            project_configs,
            db_pool,
        })
    }
}
