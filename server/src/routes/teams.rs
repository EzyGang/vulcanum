use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::routes::team_auth::TeamPrincipal;
use crate::services::teams::model::{CreateTeamRequest, UpdateTeamRequest};

pub async fn list(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let teams = state
        .teams
        .list_for_principal(&auth, state.is_single_user)
        .await?;

    Ok(HttpResponse::Ok().json(teams))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team = state
        .teams
        .get_for_principal(path.into_inner(), &auth, state.is_single_user)
        .await?;

    Ok(HttpResponse::Ok().json(team))
}

pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateTeamRequest>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team = state
        .teams
        .create_for_principal(&body.name, &auth, state.is_single_user)
        .await?;

    Ok(HttpResponse::Created().json(team))
}

pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateTeamRequest>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team = state
        .teams
        .update_for_principal(path.into_inner(), &body, &auth, state.is_single_user)
        .await?;

    Ok(HttpResponse::Ok().json(team))
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    state
        .teams
        .delete_for_principal(path.into_inner(), &auth, state.is_single_user)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn list_members(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let members = state
        .teams
        .list_members_for_principal(path.into_inner(), &auth, state.is_single_user)
        .await?;

    Ok(HttpResponse::Ok().json(members))
}

pub async fn create_invite(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let invite = state
        .teams
        .create_invite_for_principal(path.into_inner(), &auth, state.is_single_user)
        .await?;

    Ok(HttpResponse::Created().json(invite))
}

pub async fn preview_invite(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let invite = state
        .teams
        .preview_invite(&path.into_inner(), state.is_single_user)
        .await?;

    Ok(HttpResponse::Ok().json(invite))
}

pub async fn accept_invite(
    state: web::Data<AppState>,
    path: web::Path<String>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let invite = state
        .teams
        .accept_invite_for_principal(&path.into_inner(), &auth, state.is_single_user)
        .await?;

    Ok(HttpResponse::Ok().json(invite))
}
