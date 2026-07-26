use actix_web::HttpResponse;

use vulcanum_shared::{
    api::wire::StatusResponse,
    constants::{
        APP_ACCESS_TOKEN_TTL_MINUTES, APP_REFRESH_TOKEN_TTL_DAYS, WORKER_ACCESS_TOKEN_TTL_MINUTES,
        WORKER_REFRESH_TOKEN_TTL_DAYS, WORKER_REGISTRATION_CODE_TTL_MINUTES,
    },
};

pub async fn get() -> HttpResponse {
    HttpResponse::Ok().json(StatusResponse {
        app_access_token_ttl_minutes: APP_ACCESS_TOKEN_TTL_MINUTES,
        app_refresh_token_ttl_days: APP_REFRESH_TOKEN_TTL_DAYS,
        worker_access_token_ttl_minutes: WORKER_ACCESS_TOKEN_TTL_MINUTES,
        worker_refresh_token_ttl_days: WORKER_REFRESH_TOKEN_TTL_DAYS,
        worker_registration_code_ttl_minutes: WORKER_REGISTRATION_CODE_TTL_MINUTES,
    })
}
