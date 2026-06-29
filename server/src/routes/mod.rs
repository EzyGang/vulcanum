pub mod auth;
pub mod github;
pub mod health;
pub mod instance_auth;
pub mod jobs;
pub mod model_providers;
pub mod project_configs;
pub mod providers;
pub mod status;
pub mod task_board;
pub mod team_auth;
pub mod teams;
pub mod work_runs;
pub mod worker_auth;
pub mod worker_or_instance_auth;
pub mod workers;

use actix_web::web;
use actix_web::HttpRequest;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/auth/login", web::post().to(auth::login))
            .route("/auth/verify", web::get().to(auth::verify))
            .route("/auth/mode", web::get().to(auth::mode))
            .route("/auth/me", web::get().to(auth::me))
            .route("/auth/exchange", web::post().to(auth::exchange))
            .route("/auth/refresh", web::post().to(auth::refresh))
            .route("/auth/github/start", web::get().to(auth::github_start))
            .route(
                "/auth/github/callback",
                web::get().to(auth::github_callback),
            )
            .route("/auth/instance-login", web::post().to(auth::instance_login))
            .route("/auth/logout", web::post().to(auth::logout))
            .service(
                web::scope("/teams")
                    .route("", web::get().to(teams::list))
                    .route("", web::post().to(teams::create))
                    .route("/defaults", web::get().to(teams::defaults))
                    .route("/{id}", web::get().to(teams::get))
                    .route("/{id}", web::patch().to(teams::update))
                    .route("/{id}", web::delete().to(teams::delete))
                    .route("/{id}/invites", web::post().to(teams::create_invite))
                    .route("/{id}/members", web::get().to(teams::list_members)),
            )
            .service(
                web::scope("/team-invites")
                    .route("/{token}", web::get().to(teams::preview_invite))
                    .route("/{token}/accept", web::post().to(teams::accept_invite)),
            )
            .service(
                web::scope("/github")
                    .route("/auth", web::get().to(github::auth_redirect))
                    .route("/auth-url", web::get().to(github::auth_url))
                    .route("/callback", web::get().to(github::callback))
                    .route("/repos", web::get().to(github::list_repos))
                    .route("/installation", web::get().to(github::get_installation))
                    .route(
                        "/installation/{id}",
                        web::delete().to(github::delete_installation),
                    ),
            )
            .route("/poll", web::get().to(jobs::poll))
            .route("/health", web::get().to(health::get))
            .route("/status", web::get().to(status::get))
            .service(
                web::scope("/jobs")
                    .route("/{id}", web::get().to(jobs::get_job))
                    .route(
                        "/{id}/github-token",
                        web::post().to(jobs::refresh_github_token),
                    )
                    .route("/{id}/ack", web::post().to(jobs::ack_job))
                    .route("/{id}/result", web::post().to(jobs::submit_result))
                    .route("/{id}/events", web::post().to(jobs::append_events))
                    .route("/{id}/events", web::get().to(jobs::list_events)),
            )
            .service(
                web::scope("/projects")
                    .route("", web::get().to(project_configs::list))
                    .route("", web::post().to(project_configs::create))
                    .route("/stats", web::get().to(project_configs::stats))
                    .route("/{id}", web::get().to(project_configs::get))
                    .route("/{id}", web::patch().to(project_configs::update))
                    .route("/{id}", web::delete().to(project_configs::delete)),
            )
            .service(
                web::scope("/model-providers")
                    .route("/catalog", web::get().to(model_providers::catalog))
                    .route(
                        "/device-flows",
                        web::post().to(model_providers::start_device_flow),
                    )
                    .route(
                        "/device-flows/{attempt_id}/poll",
                        web::post().to(model_providers::poll_device_flow),
                    )
                    .route("", web::get().to(model_providers::list))
                    .route("", web::post().to(model_providers::create))
                    .route("/{id}", web::patch().to(model_providers::update))
                    .route("/{id}", web::delete().to(model_providers::delete)),
            )
            .service(
                web::scope("/providers")
                    .route("", web::get().to(providers::list))
                    .route("", web::post().to(providers::create))
                    .route("/{id}", web::get().to(providers::get))
                    .route("/{id}", web::patch().to(providers::update))
                    .route("/{id}", web::delete().to(providers::delete))
                    .route(
                        "/{id}/projects/lookup",
                        web::get().to(providers::lookup_project),
                    )
                    .route(
                        "/{id}/workspaces",
                        web::get().to(providers::list_workspaces),
                    )
                    .route("/{id}/projects", web::get().to(providers::list_projects)),
            )
            .service(
                web::scope("/task-board")
                    .route("/projects", web::get().to(task_board::list_projects))
                    .route(
                        "/providers/{provider_id}/projects/{project_id}",
                        web::get().to(task_board::get_board),
                    )
                    .route(
                        "/providers/{provider_id}/projects/{project_id}/tasks",
                        web::post().to(task_board::create_task),
                    )
                    .route(
                        "/providers/{provider_id}/tasks/{task_id}",
                        web::patch().to(task_board::update_task),
                    )
                    .route(
                        "/providers/{provider_id}/tasks/{task_id}/status",
                        web::patch().to(task_board::move_task),
                    )
                    .route(
                        "/providers/{provider_id}/tasks/{task_id}/labels/{label_id}",
                        web::put().to(task_board::add_task_label),
                    )
                    .route(
                        "/providers/{provider_id}/tasks/{task_id}/labels/{label_id}",
                        web::delete().to(task_board::remove_task_label),
                    ),
            )
            .service(
                web::scope("/workers")
                    .route("", web::get().to(workers::list))
                    .route("/codes", web::post().to(workers::generate_code))
                    .route("/connect", web::post().to(workers::connect))
                    .route("/refresh", web::post().to(workers::refresh))
                    .route("/me", web::delete().to(workers::self_delete))
                    .route("/{id}/status", web::patch().to(workers::update_status))
                    .route("/{id}", web::delete().to(workers::delete)),
            )
            .service(
                web::scope("/runs")
                    .route("", web::get().to(work_runs::list))
                    .route("/bulk-delete", web::post().to(work_runs::bulk_delete))
                    .route("/{id}", web::delete().to(work_runs::delete))
                    .route("/{id}/fail", web::post().to(work_runs::fail_run))
                    .route("/{id}/cancel", web::post().to(work_runs::cancel_run))
                    .route(
                        "/{id}/events/recent",
                        web::get().to(work_runs::list_events_recent),
                    ),
            ),
    );
}

fn parse_team_header(req: &HttpRequest) -> Result<Option<Uuid>, AppError> {
    let header = match req.headers().get("X-Team-Id") {
        Some(header) => header,
        None => return Ok(None),
    };

    let value = header
        .to_str()
        .map_err(|_| AppError::BadRequest("Invalid X-Team-Id header".to_owned()))?;
    Uuid::parse_str(value)
        .map(Some)
        .map_err(|_| AppError::BadRequest("Invalid X-Team-Id header".to_owned()))
}

fn decode_jwt<T>(req: &HttpRequest, missing_header_error: AppError) -> Result<T, AppError>
where
    T: DeserializeOwned,
{
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(missing_header_error)?;

    let state = req
        .app_data::<web::Data<AppState>>()
        .ok_or(AppError::Internal)?;
    let decoding_key = jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes());
    let validation = jsonwebtoken::Validation::default();

    jsonwebtoken::decode::<T>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|_| AppError::InvalidToken)
}

#[cfg(test)]
mod auth_tests;

#[cfg(test)]
mod project_configs_tests;

#[cfg(test)]
mod teams_tests;

#[cfg(test)]
mod workers_tests;

#[cfg(test)]
mod jobs_tests;
