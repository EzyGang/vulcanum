use actix_web::HttpResponse;
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    ping: String,
}

pub async fn get() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        ping: "pong".to_owned(),
    })
}
