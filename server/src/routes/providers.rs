use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::routes::team_auth::TeamPrincipal;
use crate::services::project_configs::model::{LookupProjectResult, ProjectInfo, WorkspaceInfo};
use crate::services::provider_configs::model::{CreateProviderRequest, UpdateProviderRequest};

pub async fn list(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let providers = state.providers.list_all(team_id).await?;

    Ok(HttpResponse::Ok().json(providers))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let provider = state.providers.get_by_id(id, team_id).await?;

    Ok(HttpResponse::Ok().json(provider))
}

pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateProviderRequest>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let provider = state.providers.create(team_id, body.into_inner()).await?;

    Ok(HttpResponse::Created().json(provider))
}

pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateProviderRequest>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let provider = state
        .providers
        .update(id, team_id, body.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(provider))
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    state.providers.delete(id, team_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn lookup_project(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<LookupQuery>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let provider_id = path.into_inner();
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let result: LookupProjectResult = state
        .project_configs
        .lookup_project(&provider_id, team_id, &query.external_project_id)
        .await?;

    Ok(HttpResponse::Ok().json(result))
}

pub async fn list_workspaces(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let provider_id = path.into_inner();
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let workspaces: Vec<WorkspaceInfo> = state
        .project_configs
        .fetch_workspaces(&provider_id, team_id)
        .await?;

    Ok(HttpResponse::Ok().json(workspaces))
}

pub async fn list_projects(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<ListProjectsQuery>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let provider_id = path.into_inner();
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let projects: Vec<ProjectInfo> = state
        .project_configs
        .fetch_projects(&provider_id, team_id, &query.workspace_id)
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
