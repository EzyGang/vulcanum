use actix_web::HttpResponse;

use crate::models::workers::model;
use vulcanum_shared::api_types::StatusResponse;

pub async fn get() -> HttpResponse {
    HttpResponse::Ok().json(StatusResponse {
        access_token_ttl_minutes: model::ACCESS_TOKEN_TTL_MINUTES,
        code_ttl_minutes: model::CODE_TTL_MINUTES,
        refresh_token_ttl_days: model::REFRESH_TOKEN_TTL_DAYS,
    })
}
