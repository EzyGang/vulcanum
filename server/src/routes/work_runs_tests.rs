use actix_web::{test, web, App};

use crate::routes;
use crate::test_helpers;

#[sqlx::test]
async fn list_runs_rejects_negative_pagination(pool: sqlx::PgPool) {
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

    for (uri, expected_error) in [
        ("/api/v1/runs?limit=-1", "limit must be greater than zero"),
        (
            "/api/v1/runs?offset=-1",
            "offset must be greater than or equal to zero",
        ),
    ] {
        let req = test::TestRequest::get()
            .uri(uri)
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 400, "{uri}");
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["error"], expected_error, "{uri}");
    }
}
