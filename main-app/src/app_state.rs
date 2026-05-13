use sqlx::PgPool;

use crate::config::AppConfig;
use crate::services::auth::service::AuthService;
use crate::services::users::repository::UsersRepository;
use crate::services::users::service::UsersService;

#[derive(Clone)]
pub struct AppState {
    pub auth: AuthService,
    pub db_pool: PgPool,
}

impl AppState {
    pub async fn new(cfg: &AppConfig) -> Result<Self, sqlx::Error> {
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(cfg.max_conns)
            .connect(&cfg.db_url)
            .await?;

        let users = UsersService::new(UsersRepository::new(), db_pool.clone());
        let auth = AuthService::new(users);

        Ok(Self { auth, db_pool })
    }
}
