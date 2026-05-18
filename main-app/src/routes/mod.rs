pub mod auth;
pub mod project_configs;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/auth/login", web::post().to(auth::login))
            .route("/auth/verify", web::get().to(auth::verify))
            .service(
                web::scope("/projects")
                    .route("", web::get().to(project_configs::list))
                    .route("", web::post().to(project_configs::create))
                    .route("/{id}", web::get().to(project_configs::get))
                    .route("/{id}", web::put().to(project_configs::update))
                    .route("/{id}", web::delete().to(project_configs::delete))
                    .route(
                        "/{id}/columns",
                        web::post().to(project_configs::list_columns),
                    ),
            ),
    );
}

#[cfg(test)]
mod project_configs_tests;
