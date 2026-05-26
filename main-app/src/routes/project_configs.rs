use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::routes::instance_auth::InstanceAuth;
use crate::services::project_configs::model::{
    ColumnInfo, CreateProjectConfigRequest, UpdateProjectConfigRequest,
};

pub async fn list(
    state: web::Data<AppState>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let configs = state.project_configs.list_all().await?;

    Ok(HttpResponse::Ok().json(configs))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let config = state.project_configs.get_by_id(id).await?;

    Ok(HttpResponse::Ok().json(config))
}

pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateProjectConfigRequest>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let config = state.project_configs.create(body.into_inner()).await?;

    Ok(HttpResponse::Created().json(config))
}

pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateProjectConfigRequest>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let config = state.project_configs.update(id, body.into_inner()).await?;

    Ok(HttpResponse::Ok().json(config))
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    state.project_configs.delete(id).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize)]
pub struct ColumnsResponse {
    pub columns: Vec<ColumnInfo>,
}

pub async fn list_columns(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let columns = state.project_configs.fetch_columns(id).await?;

    Ok(HttpResponse::Ok().json(ColumnsResponse { columns }))
}

#[derive(Deserialize)]
pub struct ColumnsQuery {
    pub kaneo_project_id: String,
}

pub async fn list_columns_by_kaneo_id(
    state: web::Data<AppState>,
    query: web::Query<ColumnsQuery>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let columns = state
        .project_configs
        .fetch_columns_by_kaneo_id(&query.kaneo_project_id)
        .await?;

    Ok(HttpResponse::Ok().json(ColumnsResponse { columns }))
}

#[derive(Serialize)]
pub struct ProjectStats {
    pub enabled_count: i64,
}

pub async fn stats(
    state: web::Data<AppState>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let enabled_count = state.project_configs.count_enabled().await?;

    Ok(HttpResponse::Ok().json(ProjectStats { enabled_count }))
}
