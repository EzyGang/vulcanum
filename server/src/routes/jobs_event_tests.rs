use actix_web::{test, web, App};
use uuid::Uuid;

use crate::routes;
use crate::test_helpers;

#[sqlx::test]
async fn list_job_events_rejects_after_sequence_above_i64_max(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool).await;
    let token = state
        .auth
        .instance_login("test-password")
        .expect("should issue instance token");
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;
    let id = Uuid::new_v4();
    let too_large = i64::MAX as u64 + 1;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/jobs/{id}/events?after_sequence={too_large}"
        ))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "after_sequence is too large");
}
