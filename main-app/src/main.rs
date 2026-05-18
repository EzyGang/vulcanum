mod app_state;
mod config;
mod errors;
mod routes;
mod services;

use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing_log::LogTracer::init().ok();

    let cfg = config::config();
    let app_state = app_state::AppState::new(cfg).await?;

    log::info!("Applying migrations...");
    sqlx::migrate!().run(&app_state.db_pool).await?;

    let poller = app_state.clone().into_poller(cfg.poll_period_secs);
    tokio::spawn(poller.run());

    log::info!("Starting server on 0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(routes::configure)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    Ok(())
}
