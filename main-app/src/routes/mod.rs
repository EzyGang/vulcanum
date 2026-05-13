pub mod auth;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/auth/login", web::post().to(auth::login))
            .route("/auth/verify", web::get().to(auth::verify)),
    );
}
