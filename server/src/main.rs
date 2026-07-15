use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use tokio_util::sync::CancellationToken;
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
    let webhook_service = app_state.github_webhooks.clone();

    tracing::info!("Starting server on 0.0.0.0:8000");
    let server = HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            .configure(vulcanum_server::routes::configure)
    })
    .bind("0.0.0.0:8000")?
    .run();
    let cancellation = CancellationToken::new();
    let webhook_worker = tokio::spawn(webhook_service.run(cancellation.child_token()));
    let server_result = server.await;

    cancellation.cancel();
    match webhook_worker.await {
        Ok(()) => (),
        Err(e) => tracing::error!(error = %e, "GitHub webhook worker failed to shut down"),
    }
    server_result?;

    Ok(())
}
