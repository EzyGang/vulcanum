use super::*;

#[sqlx::test]
async fn connect_refresh_poll_no_jobs(pool: sqlx::PgPool) {
    let state = test_helpers::state::build_state(pool.clone()).await;
    let token = state
        .auth
        .instance_login("test-password")
        .await
        .unwrap()
        .access_token;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .configure(routes::configure),
    )
    .await;

    let code_req = test::TestRequest::post()
        .uri("/api/v1/workers/codes")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let code_resp = test::call_service(&app, code_req).await;
    assert_eq!(code_resp.status(), 201);
    let code_body: serde_json::Value = test::read_body_json(code_resp).await;
    let code = code_body["code"].as_str().unwrap();

    let connect_req = test::TestRequest::post()
        .uri("/api/v1/workers/connect")
        .set_json(json!({"code": code, "worker_name": "e2e-worker"}))
        .to_request();
    let connect_resp = test::call_service(&app, connect_req).await;
    assert_eq!(connect_resp.status(), 200);
    let connect_body: serde_json::Value = test::read_body_json(connect_resp).await;
    let refresh_token = connect_body["refresh_token"].as_str().unwrap();
    let worker_id = connect_body["worker_id"].as_str().unwrap();
    let worker_uuid = Uuid::parse_str(worker_id).unwrap();

    let refresh_req = test::TestRequest::post()
        .uri("/api/v1/workers/refresh")
        .set_json(json!({"refresh_token": refresh_token}))
        .to_request();
    let refresh_resp = test::call_service(&app, refresh_req).await;
    assert_eq!(refresh_resp.status(), 200);
    let refresh_body: serde_json::Value = test::read_body_json(refresh_resp).await;
    let access_token = refresh_body["access_token"].as_str().unwrap();

    let poll_req = test::TestRequest::get()
        .uri("/api/v1/poll")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let poll_resp = test::call_service(&app, poll_req).await;
    assert_eq!(poll_resp.status(), 204);

    let row = sqlx::query!("SELECT last_seen FROM workers WHERE id = $1", worker_uuid)
        .fetch_one(&pool)
        .await
        .expect("Should query worker");
    assert!(
        row.last_seen.is_some(),
        "last_seen should be updated after poll"
    );
}
