use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::routes::team_auth::TeamPrincipal;
use crate::services::model_providers::model::{
    CreateModelProviderRequest, StartChatGptAuthRequest, UpdateModelProviderRequest,
};

pub async fn catalog(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let catalog = state.model_providers.catalog().await?;
    Ok(HttpResponse::Ok().json(catalog))
}

pub async fn list(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let providers = state.model_providers.list_all(team_id).await?;
    Ok(HttpResponse::Ok().json(providers))
}

pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateModelProviderRequest>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team_for_owner(&auth, state.is_single_user)
        .await?;
    let provider = state
        .model_providers
        .create(team_id, body.into_inner())
        .await?;
    Ok(HttpResponse::Created().json(provider))
}

pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateModelProviderRequest>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team_for_owner(&auth, state.is_single_user)
        .await?;
    let provider = state
        .model_providers
        .update(path.into_inner(), team_id, body.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(provider))
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team_for_owner(&auth, state.is_single_user)
        .await?;
    state
        .model_providers
        .delete(path.into_inner(), team_id)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn start_chatgpt_auth(
    state: web::Data<AppState>,
    body: web::Json<StartChatGptAuthRequest>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team_for_owner(&auth, state.is_single_user)
        .await?;
    let user_id = principal_user_id(&auth);
    let response = state
        .model_providers
        .start_chatgpt_auth(team_id, &user_id, body.into_inner())
        .await?;
    Ok(HttpResponse::Created().json(response))
}

pub async fn chatgpt_auth_status(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let user_id = principal_user_id(&auth);
    let response = state
        .model_providers
        .chatgpt_auth_status(team_id, &user_id, path.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn cancel_chatgpt_auth(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let user_id = principal_user_id(&auth);
    state
        .model_providers
        .cancel_chatgpt_auth(team_id, &user_id, path.into_inner())
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

fn principal_user_id(auth: &TeamPrincipal) -> String {
    match auth {
        TeamPrincipal::User { user_id, .. } => user_id.clone(),
        TeamPrincipal::Instance { .. } => "instance".to_owned(),
    }
}
