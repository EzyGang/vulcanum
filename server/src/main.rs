use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use tracing_actix_web::TracingLogger;

use vulcanum_server::app_state;
use vulcanum_server::config;

#[actix_web::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();

    vulcanum_shared::telemetry::init();

    let cfg = config::config()?;
    let app_state = app_state::AppState::new(cfg).await?;

    tracing::info!("Applying migrations...");
    app_state.run_migrations().await?;

    let poller = app_state.clone().into_poller(cfg.poll_period_secs);
    tokio::spawn(poller.run());
    tokio::spawn(app_state.github_webhooks.clone().run());

    tracing::info!("Starting server on 0.0.0.0:8000");
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            .configure(vulcanum_server::routes::configure)
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await?;

    Ok(())
}
