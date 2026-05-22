mod app_state;
mod config;
mod errors;
mod queryer;
mod routes;
mod services;

#[cfg(test)]
mod test_helpers;

use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();

    vulcanum_shared::telemetry::init();

    let cfg = config::config();
    let app_state = app_state::AppState::new(cfg).await?;

    tracing::info!("Applying migrations...");
    sqlx::migrate!().run(&app_state.db_pool).await?;

    let poller = app_state.clone().into_poller(cfg.poll_period_secs);
    tokio::spawn(poller.run());

    tracing::info!("Starting server on 0.0.0.0:8080");
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
