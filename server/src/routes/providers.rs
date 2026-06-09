use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::routes::instance_auth::InstanceAuth;
use crate::services::project_configs::model::{LookupProjectResult, ProjectInfo, WorkspaceInfo};
use crate::services::provider_configs::model::{CreateProviderRequest, UpdateProviderRequest};

pub async fn list(
    state: web::Data<AppState>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let providers = state.providers.list_all().await?;

    Ok(HttpResponse::Ok().json(providers))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let provider = state.providers.get_by_id(id).await?;

    Ok(HttpResponse::Ok().json(provider))
}

pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateProviderRequest>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let provider = state.providers.create(body.into_inner()).await?;

    Ok(HttpResponse::Created().json(provider))
}

pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateProviderRequest>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let provider = state.providers.update(id, body.into_inner()).await?;

    Ok(HttpResponse::Ok().json(provider))
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    state.providers.delete(id).await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn lookup_project(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<LookupQuery>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let provider_id = path.into_inner();
    let result: LookupProjectResult = state
        .project_configs
        .lookup_project(&provider_id, &query.external_project_id)
        .await?;

    Ok(HttpResponse::Ok().json(result))
}

pub async fn list_workspaces(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let provider_id = path.into_inner();
    let workspaces: Vec<WorkspaceInfo> =
        state.project_configs.fetch_workspaces(&provider_id).await?;

    Ok(HttpResponse::Ok().json(workspaces))
}

pub async fn list_projects(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<ListProjectsQuery>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let provider_id = path.into_inner();
    let projects: Vec<ProjectInfo> = state
        .project_configs
        .fetch_projects(&provider_id, &query.workspace_id)
        .await?;

    Ok(HttpResponse::Ok().json(projects))
}

#[derive(serde::Deserialize)]
pub struct LookupQuery {
    pub external_project_id: String,
}

#[derive(serde::Deserialize)]
pub struct ListProjectsQuery {
    pub workspace_id: String,
}
