use actix_web::{test, web, App};
use uuid::Uuid;

use crate::models::providers::model::UpdateIntegrationTaskInput;
use crate::routes;
use crate::services::providers::kaneo::client::KaneoClient;
use crate::test_helpers;
use crate::test_helpers::kaneo::{start_kaneo_server, SslCertFileGuard, SSL_CERT_FILE_LOCK};

async fn insert_provider(pool: &sqlx::PgPool, instance_url: &str) -> Uuid {
    let id = Uuid::new_v4();
    test_helpers::teams::ensure_default_team(pool).await;

    sqlx::query!(
        "INSERT INTO integration_providers (id, team_id, name, instance_url, api_key) VALUES ($1, $2, $3, $4, $5)",
        id,
        test_helpers::DEFAULT_TEAM_ID,
        "task-board-route-provider",
        instance_url,
        "test-key",
    )
    .execute(pool)
    .await
    .expect("insert task board provider");

    id
}

#[actix_web::test]
async fn kaneo_client_captures_full_update_over_tls() {
    let _ssl_cert_file_guard = SSL_CERT_FILE_LOCK.lock().await;
    let server = start_kaneo_server(200).await;
    let _ssl_cert_file = SslCertFileGuard::set(&server.certificate_path);

    KaneoClient::new(server.instance_url, "test-key".to_owned())
        .update_task(&UpdateIntegrationTaskInput {
            task_id: "task-1".to_owned(),
            title: "Current title".to_owned(),
            body: "Updated body".to_owned(),
            status: "in-progress".to_owned(),
            priority: "urgent".to_owned(),
            project_id: "project-1".to_owned(),
            position: 7.5,
            due_date: Some("2026-01-10T00:00:00Z".to_owned()),
            start_date: Some("2026-01-03T00:00:00Z".to_owned()),
            user_id: Some("user-1".to_owned()),
        })
        .await
        .expect("Kaneo task update succeeds");

    let payload = server.request.await.expect("captured Kaneo PUT request");
    assert_eq!(payload["description"], "Updated body");
    assert_eq!(payload["status"], "in-progress");
    assert_eq!(payload["priority"], "urgent");
}

#[actix_web::test]
async fn kaneo_client_fetches_task_over_tls() {
    let _ssl_cert_file_guard = SSL_CERT_FILE_LOCK.lock().await;
    let server = start_kaneo_server(200).await;
    let _ssl_cert_file = SslCertFileGuard::set(&server.certificate_path);

    let task = KaneoClient::new(server.instance_url, "test-key".to_owned())
        .fetch_task("task-1")
        .await
        .expect("Kaneo task fetch succeeds");

    assert_eq!(task.id, "task-1");
    assert_eq!(task.project_id, "project-1");
}

#[sqlx::test]
async fn task_edit_fetches_current_task_and_sends_full_kaneo_update(pool: sqlx::PgPool) {
    let _ssl_cert_file_guard = SSL_CERT_FILE_LOCK.lock().await;
    let server = start_kaneo_server(200).await;
    let provider_id = insert_provider(&pool, &server.instance_url).await;
    test_helpers::project_configs::insert_project_config_with_provider(
        &pool,
        "project-1",
        provider_id,
    )
    .await;

    let state = test_helpers::state::build_state(pool).await;
    let token = state
        .auth
        .instance_login("test-password")
        .await
        .expect("instance login")
        .access_token;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let _ssl_cert_file = SslCertFileGuard::set(&server.certificate_path);
    let response = test::call_service(
        &app,
        test::TestRequest::patch()
            .uri(&format!(
                "/api/v1/task-board/providers/{provider_id}/tasks/task-1"
            ))
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(serde_json::json!({ "title": "Current title", "body": "Updated body" }))
            .to_request(),
    )
    .await;

    assert!(response.status().is_success());
    let payload = server.request.await.expect("captured Kaneo PUT request");
    assert_eq!(
        payload,
        serde_json::json!({
            "title": "Current title",
            "description": "Updated body",
            "priority": "urgent",
            "status": "in-progress",
            "projectId": "project-1",
            "position": 7.5,
            "dueDate": "2026-01-10T00:00:00Z",
            "startDate": "2026-01-03T00:00:00Z",
            "userId": "user-1"
        })
    );
}

#[sqlx::test]
async fn task_edit_sanitizes_kaneo_validation_error(pool: sqlx::PgPool) {
    let _ssl_cert_file_guard = SSL_CERT_FILE_LOCK.lock().await;
    let server = start_kaneo_server(400).await;
    let provider_id = insert_provider(&pool, &server.instance_url).await;
    test_helpers::project_configs::insert_project_config_with_provider(
        &pool,
        "project-1",
        provider_id,
    )
    .await;

    let state = test_helpers::state::build_state(pool).await;
    let token = state
        .auth
        .instance_login("test-password")
        .await
        .expect("instance login")
        .access_token;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let _ssl_cert_file = SslCertFileGuard::set(&server.certificate_path);
    let response = test::call_service(
        &app,
        test::TestRequest::patch()
            .uri(&format!(
                "/api/v1/task-board/providers/{provider_id}/tasks/task-1"
            ))
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(serde_json::json!({ "title": "Current title", "body": "Updated body" }))
            .to_request(),
    )
    .await;

    assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(
        payload["error"],
        "Task tracker rejected the update. Refresh the board and retry"
    );
    assert!(!payload["error"]
        .as_str()
        .expect("error is a string")
        .contains("secret body"));
    let _ = server.request.await.expect("captured Kaneo PUT request");
}
