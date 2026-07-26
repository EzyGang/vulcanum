use actix_web::{body::to_bytes, http::StatusCode};
use vulcanum_shared::api::wire::StatusResponse;

use crate::routes::status;

#[actix_web::test]
async fn reports_app_session_token_lifetimes() {
    let response = status::get().await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body())
        .await
        .expect("read status response");
    let status: StatusResponse = serde_json::from_slice(&body).expect("decode status response");
    assert_eq!(status.access_token_ttl_minutes, 15);
    assert_eq!(status.refresh_token_ttl_days, 30);
}
