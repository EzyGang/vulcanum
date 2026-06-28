use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::models::auth::model::TeamPrincipal;
use crate::models::task_board::model::{CreateTaskRequest, MoveTaskRequest};

pub async fn list_projects(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let projects = state
        .task_board
        .list_projects(&state.project_configs.db, team_id)
        .await?;

    Ok(HttpResponse::Ok().json(projects))
}

pub async fn get_board(
    state: web::Data<AppState>,
    path: web::Path<(Uuid, String)>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let (provider_id, external_project_id) = path.into_inner();
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let board = state
        .task_board
        .get_board(
            &state.project_configs.db,
            team_id,
            provider_id,
            &external_project_id,
        )
        .await?;

    Ok(HttpResponse::Ok().json(board))
}

pub async fn create_task(
    state: web::Data<AppState>,
    path: web::Path<(Uuid, String)>,
    body: web::Json<CreateTaskRequest>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let (provider_id, external_project_id) = path.into_inner();
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let task = state
        .task_board
        .create_task(
            &state.project_configs.db,
            team_id,
            provider_id,
            &external_project_id,
            body.into_inner(),
        )
        .await?;

    Ok(HttpResponse::Created().json(task))
}

pub async fn move_task(
    state: web::Data<AppState>,
    path: web::Path<(Uuid, String)>,
    body: web::Json<MoveTaskRequest>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let (provider_id, task_id) = path.into_inner();
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let result = state
        .task_board
        .move_task(
            &state.project_configs.db,
            team_id,
            provider_id,
            &task_id,
            &body.status,
        )
        .await?;

    Ok(HttpResponse::Ok().json(result))
}
