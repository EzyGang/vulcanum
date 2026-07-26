use super::*;

#[sqlx::test]
async fn poll_rejects_missing_auth(pool: sqlx::PgPool) {
    let state = build_state(pool).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v1/poll").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 401);
}

#[sqlx::test]
async fn poll_rejects_invalid_token(pool: sqlx::PgPool) {
    let state = build_state(pool).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/poll")
        .insert_header(("Authorization", "Bearer garbage"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 401);
}
