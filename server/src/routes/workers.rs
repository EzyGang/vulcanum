use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::routes::team_auth::TeamPrincipal;
use crate::services::workers::model::{ConnectRequest, RefreshRequest, UpdateWorkerStatusRequest};

pub async fn generate_code(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let resp = state.workers.generate_code(team_id).await?;
    Ok(HttpResponse::Created().json(resp))
}

pub async fn connect(
    state: web::Data<AppState>,
    body: web::Json<ConnectRequest>,
) -> Result<HttpResponse, AppError> {
    let resp = state.workers.connect(body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(resp))
}

pub async fn refresh(
    state: web::Data<AppState>,
    body: web::Json<RefreshRequest>,
) -> Result<HttpResponse, AppError> {
    let resp = state.workers.refresh(body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(resp))
}

pub async fn list(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let workers = state.workers.list_all(team_id).await?;
    Ok(HttpResponse::Ok().json(workers))
}

pub async fn update_status(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateWorkerStatusRequest>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let worker = state
        .workers
        .set_worker_status(path.into_inner(), team_id, body.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(worker))
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    state
        .workers
        .delete_worker(path.into_inner(), team_id)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}
