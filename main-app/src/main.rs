mod config;
mod db;
mod errors;
mod models;
mod routes;

use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let cfg = config::config();
    db::db_config::init_pool(&cfg.db_url, cfg.max_conns).await?;

    log::info!("Applying migrations...");
    sqlx::migrate!().run(db::db_config::pool()).await?;

    log::info!("Starting server on 0.0.0.0:8080");
    HttpServer::new(|| App::new().configure(routes::configure))
        .bind("0.0.0.0:8080")?
        .run()
        .await?;

    Ok(())
}
