use actix_web::HttpResponse;

use vulcanum_shared::api::wire::StatusResponse;

use crate::models::{auth::model as auth_model, workers::model as worker_model};

pub async fn get() -> HttpResponse {
    HttpResponse::Ok().json(StatusResponse {
        access_token_ttl_minutes: auth_model::ACCESS_TOKEN_TTL_MINUTES,
        code_ttl_minutes: worker_model::CODE_TTL_MINUTES,
        refresh_token_ttl_days: auth_model::REFRESH_TOKEN_TTL_DAYS,
    })
}
