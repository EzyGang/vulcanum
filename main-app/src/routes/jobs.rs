use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::services::work_runs::service::work_runs::SubmitResultParams;

#[derive(Deserialize)]
pub struct PollQuery {
    worker_id: Uuid,
}

#[derive(Serialize)]
struct PollResponse {
    job_id: Uuid,
}

#[derive(Deserialize)]
pub struct SubmitResultBody {
    pr_url: String,
    exit_code: i32,
    tokens_used: i32,
    duration_ms: i32,
}

pub async fn poll(
    state: web::Data<AppState>,
    query: web::Query<PollQuery>,
) -> Result<HttpResponse, AppError> {
    match state.jobs.poll(query.worker_id).await {
        Ok(Some(job_id)) => Ok(HttpResponse::Ok().json(PollResponse { job_id })),
        Ok(None) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Err(e.into()),
    }
}

pub async fn get_job(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let job = state.jobs.get_job(path.into_inner()).await?;

    Ok(HttpResponse::Ok().json(job))
}

pub async fn ack_job(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<AckBody>,
) -> Result<HttpResponse, AppError> {
    let job = state
        .jobs
        .ack_job(path.into_inner(), body.worker_id)
        .await?;

    Ok(HttpResponse::Ok().json(job))
}

pub async fn submit_result(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<SubmitResultBody>,
) -> Result<HttpResponse, AppError> {
    let b = body.into_inner();
    let params = SubmitResultParams {
        pr_url: b.pr_url,
        exit_code: b.exit_code,
        tokens_used: b.tokens_used,
        duration_ms: b.duration_ms,
    };
    let job = state.jobs.submit_result(path.into_inner(), params).await?;

    Ok(HttpResponse::Ok().json(job))
}

#[derive(Deserialize)]
pub struct AckBody {
    worker_id: Uuid,
}
