use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::routes::worker_auth::WorkerAuth;
use crate::routes::worker_or_instance_auth::WorkerOrInstanceAuth;
use vulcanum_shared::api_types::{
    AckRequest, AppendEventsRequest, AppendEventsResponse, ListEventsResponse, PollResponse,
    SubmitResultRequest,
};

pub async fn poll(state: web::Data<AppState>, auth: WorkerAuth) -> Result<HttpResponse, AppError> {
    match state.jobs.poll(auth.worker_id).await {
        Ok(Some(job_id)) => Ok(HttpResponse::Ok().json(PollResponse { job_id })),
        Ok(None) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Err(e.into()),
    }
}

pub async fn get_job(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    _auth: WorkerAuth,
) -> Result<HttpResponse, AppError> {
    let job = state.jobs.get_job(path.into_inner()).await?;

    Ok(HttpResponse::Ok().json(job))
}

pub async fn ack_job(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    _body: web::Json<AckRequest>,
    auth: WorkerAuth,
) -> Result<HttpResponse, AppError> {
    let job = state
        .jobs
        .ack_job(path.into_inner(), auth.worker_id)
        .await?;

    Ok(HttpResponse::Ok().json(job))
}

pub async fn submit_result(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<SubmitResultRequest>,
    auth: WorkerAuth,
) -> Result<HttpResponse, AppError> {
    let job = state
        .jobs
        .submit_result(path.into_inner(), auth.worker_id, body.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(job))
}

pub async fn append_events(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<AppendEventsRequest>,
    auth: WorkerAuth,
) -> Result<HttpResponse, AppError> {
    let work_run_id = path.into_inner();
    let req = body.into_inner();

    let result = state
        .events
        .append_events(work_run_id, auth.worker_id, req.events)
        .await?;

    Ok(HttpResponse::Ok().json(AppendEventsResponse {
        accepted: result.accepted,
        next_expected_sequence: result.next_expected_sequence as u64,
        should_cancel: result.should_cancel,
    }))
}

pub async fn list_events(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<vulcanum_shared::api_types::ListEventsQuery>,
    auth: WorkerOrInstanceAuth,
) -> Result<HttpResponse, AppError> {
    let work_run_id = path.into_inner();
    let after = query.after_sequence.unwrap_or(0) as i64;
    let limit = query.limit.unwrap_or(100);

    let result = match auth {
        WorkerOrInstanceAuth::Worker { worker_id } => {
            state
                .events
                .list_events(work_run_id, worker_id, after, limit)
                .await?
        }
        WorkerOrInstanceAuth::Instance => {
            state
                .events
                .list_events_admin(work_run_id, after, limit)
                .await?
        }
    };

    let events: Vec<vulcanum_shared::api_types::WireEvent> = result
        .events
        .into_iter()
        .map(|e| vulcanum_shared::api_types::WireEvent {
            sequence: e.sequence as u64,
            event_type: e.event_type,
            payload: e.payload,
        })
        .collect();

    Ok(HttpResponse::Ok().json(ListEventsResponse {
        events,
        has_more: result.has_more,
    }))
}
