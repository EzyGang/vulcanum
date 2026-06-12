use actix_web::{web, HttpResponse};
use serde::Serialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::routes::team_auth::TeamPrincipal;
use crate::services::project_configs::model::{
    CreateProjectConfigRequest, UpdateProjectConfigRequest,
};

pub async fn list(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let configs = state.project_configs.list_all(team_id).await?;

    Ok(HttpResponse::Ok().json(configs))
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
    let config = state.project_configs.get_by_id(id, team_id).await?;

    Ok(HttpResponse::Ok().json(config))
}

pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateProjectConfigRequest>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let config = state
        .project_configs
        .create(team_id, body.into_inner())
        .await?;

    Ok(HttpResponse::Created().json(config))
}

pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateProjectConfigRequest>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let config = state
        .project_configs
        .update(id, team_id, body.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(config))
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
    state.project_configs.delete(id, team_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize)]
pub struct ProjectStats {
    pub enabled_count: i64,
}

pub async fn stats(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let enabled_count = state.project_configs.count_enabled(team_id).await?;

    Ok(HttpResponse::Ok().json(ProjectStats { enabled_count }))
}
