use actix_web::{test, web, App};

use crate::routes;
use crate::services::teams::model::DEFAULT_REVIEW_PROMPT_TEMPLATE;
use crate::test_helpers;

#[sqlx::test]
async fn defaults_returns_review_prompt_template(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool).await;
    let token = state.auth.instance_login("test-password").unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/teams/defaults")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["review_prompt_template"].as_str(),
        Some(DEFAULT_REVIEW_PROMPT_TEMPLATE)
    );
    assert_eq!(body["max_in_progress_tasks"].as_i64(), Some(1));
}
