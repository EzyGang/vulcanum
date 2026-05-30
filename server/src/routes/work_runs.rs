use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::routes::instance_auth::InstanceAuth;
use crate::services::work_runs::model::WorkRunStatus;
use vulcanum_shared::api_types::{BulkDeleteRunsRequest, BulkDeleteRunsResponse};

#[derive(Deserialize)]
pub struct ListRunsQuery {
    pub status: Option<WorkRunStatus>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<ListRunsQuery>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);
    let runs = state.jobs.list_all(query.status, limit, offset).await?;

    Ok(HttpResponse::Ok().json(runs))
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    state.jobs.delete_run(path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn bulk_delete(
    state: web::Data<AppState>,
    body: web::Json<BulkDeleteRunsRequest>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let deleted = state.jobs.bulk_delete_runs(&body.ids).await?;
    Ok(HttpResponse::Ok().json(BulkDeleteRunsResponse { deleted }))
}

pub async fn fail_run(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let run = state.jobs.fail_run(path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(run))
}
