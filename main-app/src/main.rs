mod app_state;
mod config;
mod errors;
mod routes;
mod services;

use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let cfg = config::config();
    let app_state = app_state::AppState::new(cfg).await?;

    log::info!("Applying migrations...");
    sqlx::migrate!().run(&app_state.db_pool).await?;

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
