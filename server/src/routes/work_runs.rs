use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::routes::team_auth::TeamPrincipal;
use crate::services::work_runs::model::WorkRunStatus;
use vulcanum_shared::api_types::{BulkDeleteRunsRequest, BulkDeleteRunsResponse, WireEvent};

#[derive(Deserialize)]
pub struct ListRunsQuery {
    pub status: Option<WorkRunStatus>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<ListRunsQuery>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let runs = state
        .jobs
        .list_all(team_id, query.status, limit, offset)
        .await?;

    Ok(HttpResponse::Ok().json(runs))
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
    state.jobs.delete_run(path.into_inner(), team_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn bulk_delete(
    state: web::Data<AppState>,
    body: web::Json<BulkDeleteRunsRequest>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let deleted = state.jobs.bulk_delete_runs(&body.ids, team_id).await?;
    Ok(HttpResponse::Ok().json(BulkDeleteRunsResponse { deleted }))
}

pub async fn fail_run(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let run = state.jobs.fail_run(path.into_inner(), team_id).await?;
    Ok(HttpResponse::Ok().json(run))
}

pub async fn cancel_run(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    state.jobs.cancel_run(path.into_inner(), team_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn list_events_recent(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let work_run_id = path.into_inner();
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let events = state.events.list_recent(work_run_id, team_id).await?;

    let wire_events: Vec<WireEvent> = events
        .into_iter()
        .map(|e| WireEvent {
            sequence: e.sequence as u64,
            event_type: e.event_type,
            payload: e.payload,
            occurred_at: e.occurred_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(wire_events))
}
