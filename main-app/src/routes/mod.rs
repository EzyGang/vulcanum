pub mod auth;
pub mod jobs;
pub mod project_configs;
pub mod status;
pub mod worker_auth;
pub mod workers;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/auth/login", web::post().to(auth::login))
            .route("/auth/verify", web::get().to(auth::verify))
            .route("/poll", web::get().to(jobs::poll))
            .route("/status", web::get().to(status::get))
            .service(
                web::scope("/jobs")
                    .route("/{id}", web::get().to(jobs::get_job))
                    .route("/{id}/ack", web::post().to(jobs::ack_job))
                    .route("/{id}/result", web::post().to(jobs::submit_result)),
            )
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
            )
            .service(
                web::scope("/workers")
                    .route("/codes", web::post().to(workers::generate_code))
                    .route("/connect", web::post().to(workers::connect))
                    .route("/refresh", web::post().to(workers::refresh))
                    .route("/{id}", web::delete().to(workers::delete)),
            ),
    );
}

#[cfg(test)]
mod project_configs_tests;

#[cfg(test)]
mod workers_tests;

#[cfg(test)]
mod jobs_tests;
