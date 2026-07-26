use actix_web::{body::to_bytes, http::StatusCode};
use vulcanum_shared::api::wire::StatusResponse;

use crate::routes::status;

#[actix_web::test]
async fn reports_app_and_worker_token_lifetimes() {
    let response = status::get().await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body())
        .await
        .expect("read status response");
    let status: StatusResponse = serde_json::from_slice(&body).expect("decode status response");
    assert_eq!(status.app_access_token_ttl_minutes, 15);
    assert_eq!(status.app_refresh_token_ttl_days, 30);
    assert_eq!(status.worker_access_token_ttl_minutes, 15);
    assert_eq!(status.worker_refresh_token_ttl_days, 30);
    assert_eq!(status.worker_registration_code_ttl_minutes, 10);
}
