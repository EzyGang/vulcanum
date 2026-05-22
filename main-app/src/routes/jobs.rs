use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::routes::worker_auth::WorkerAuth;
use vulcanum_shared::api_types::{PollResponse, SubmitResultRequest};

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
