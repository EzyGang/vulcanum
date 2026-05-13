use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::OnceLock;

static POOL: OnceLock<PgPool> = OnceLock::new();

pub async fn init_pool(db_url: &str, max_conns: u32) -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(max_conns)
        .connect(db_url)
        .await?;

    POOL.set(pool)
        .map_err(|_| sqlx::Error::Configuration("DB pool already initialized".into()))
}

pub fn pool() -> &'static PgPool {
    POOL.get().expect("DB pool not initialized — call init_pool first")
}
