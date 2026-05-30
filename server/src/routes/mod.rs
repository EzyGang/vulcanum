pub mod auth;
pub mod health;
pub mod instance_auth;
pub mod jobs;
pub mod project_configs;
pub mod providers;
pub mod status;
pub mod work_runs;
pub mod worker_auth;
pub mod workers;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/auth/login", web::post().to(auth::login))
            .route("/auth/verify", web::get().to(auth::verify))
            .route("/auth/instance-login", web::post().to(auth::instance_login))
            .route("/auth/logout", web::post().to(auth::logout))
            .route("/poll", web::get().to(jobs::poll))
            .route("/health", web::get().to(health::get))
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
                    .route("/stats", web::get().to(project_configs::stats))
                    .route("/{id}", web::get().to(project_configs::get))
                    .route("/{id}", web::put().to(project_configs::update))
                    .route("/{id}", web::delete().to(project_configs::delete)),
            )
            .service(
                web::scope("/providers")
                    .route("", web::get().to(providers::list))
                    .route("", web::post().to(providers::create))
                    .route("/{id}", web::get().to(providers::get))
                    .route("/{id}", web::put().to(providers::update))
                    .route("/{id}", web::delete().to(providers::delete))
                    .route(
                        "/{id}/projects/lookup",
                        web::get().to(providers::lookup_project),
                    ),
            )
            .service(
                web::scope("/workers")
                    .route("", web::get().to(workers::list))
                    .route("/codes", web::post().to(workers::generate_code))
                    .route("/connect", web::post().to(workers::connect))
                    .route("/refresh", web::post().to(workers::refresh))
                    .route("/{id}/status", web::put().to(workers::update_status))
                    .route("/{id}", web::delete().to(workers::delete)),
            )
            .service(
                web::scope("/runs")
                    .route("", web::get().to(work_runs::list))
                    .route("/{id}", web::delete().to(work_runs::delete)),
            ),
    );
}

#[cfg(test)]
mod project_configs_tests;

#[cfg(test)]
mod workers_tests;

#[cfg(test)]
mod jobs_tests;
