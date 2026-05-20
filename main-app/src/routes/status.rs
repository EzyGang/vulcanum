use actix_web::HttpResponse;
use serde::Serialize;

use crate::services::workers::model;

#[derive(Serialize)]
struct StatusResponse {
    access_token_ttl_minutes: i64,
    code_ttl_minutes: i64,
    refresh_token_ttl_days: i64,
}

pub async fn get() -> HttpResponse {
    HttpResponse::Ok().json(StatusResponse {
        access_token_ttl_minutes: model::ACCESS_TOKEN_TTL_MINUTES,
        code_ttl_minutes: model::CODE_TTL_MINUTES,
        refresh_token_ttl_days: model::REFRESH_TOKEN_TTL_DAYS,
    })
}
