use actix_web::{test, web, App, HttpResponse};

use crate::routes::instance_auth::InstanceAuth;
use crate::test_helpers;

async fn instance_only(_: InstanceAuth) -> HttpResponse {
    HttpResponse::NoContent().finish()
}

fn signed_instance_user_token() -> String {
    let exp = chrono::Utc::now() + chrono::Duration::minutes(15);
    let claims = serde_json::json!({
        "sub": "instance",
        "typ": "user",
        "exp": exp.timestamp(),
    });
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("test-secret".as_bytes()),
    )
    .expect("should build token");
    format!("Bearer {token}")
}

#[sqlx::test]
async fn instance_auth_rejects_instance_subject_with_user_type(pool: sqlx::PgPool) {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_helpers::build_state(pool).await))
            .route("/instance-only", web::get().to(instance_only)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/instance-only")
        .insert_header(("Authorization", signed_instance_user_token()))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 401);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "Invalid token");
}
