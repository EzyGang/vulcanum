use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::routes::instance_auth::InstanceAuth;
use crate::services::workers::model::{ConnectRequest, RefreshRequest};

pub async fn generate_code(
    state: web::Data<AppState>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let resp = state.workers.generate_code().await;
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
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let workers = state.workers.list_all().await?;
    Ok(HttpResponse::Ok().json(workers))
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    state.workers.delete_worker(path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
