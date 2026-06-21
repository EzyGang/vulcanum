use actix_web::{test, web, App};
use serde_json::json;
use uuid::Uuid;

use crate::routes;
use crate::services::dispatcher::repository::DispatchRepository;
use crate::services::dispatcher::service::DispatcherService;
use crate::services::work_runs::model::{WorkRunStatus, WorkRunType};
use crate::services::work_runs::repository::queries::InsertWorkRunParams;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::services::workers::model::WorkerStatus;
use crate::test_helpers;

#[sqlx::test]
async fn review_result_with_warning_enqueues_fix_run(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool.clone()).await;
    let token = state.auth.instance_login("test-password").unwrap();

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
    let code_body: serde_json::Value = test::read_body_json(code_resp).await;
    let code = code_body["code"].as_str().unwrap();

    let connect_req = test::TestRequest::post()
        .uri("/api/v1/workers/connect")
        .set_json(json!({"code": code, "worker_name": "review-worker"}))
        .to_request();
    let connect_resp = test::call_service(&app, connect_req).await;
    let connect_body: serde_json::Value = test::read_body_json(connect_resp).await;
    let access_token = connect_body["access_token"].as_str().unwrap();
    let worker_id = connect_body["worker_id"].as_str().unwrap();
    let worker_uuid = Uuid::parse_str(worker_id).unwrap();

    let project_id = test_helpers::insert_project_config(&pool, "kaneo-review-fix").await;
    let review_run = WorkRunsRepository::new()
        .insert_work_run(
            &pool,
            InsertWorkRunParams {
                team_id: test_helpers::DEFAULT_TEAM_ID,
                external_task_ref: "task-review-warning".to_owned(),
                project_config_id: project_id,
                prompt_text: "Review the PR".to_owned(),
                repo_url: String::new(),
                repo_full_names: Vec::new(),
                agents_md: String::new(),
                status: WorkRunStatus::Pending,
                work_type: WorkRunType::PullRequestReview,
                parent_work_run_id: None,
                task_body: "Implement auth checks".to_owned(),
                task_title: Some("Auth hardening".to_owned()),
                task_slug: Some("APP-7".to_owned()),
                review_target_pr_url: Some("https://github.com/acme/app/pull/42".to_owned()),
                review_target_repo_full_name: Some("acme/app".to_owned()),
            },
        )
        .await
        .expect("review run should insert");

    DispatchRepository
        .dispatch_to_worker(&pool, review_run.id, worker_uuid)
        .await
        .expect("Should dispatch");

    state
        .dispatch_store
        .set_dispatched(worker_uuid, review_run.id)
        .await
        .expect("Should set dispatched");

    let poll_req = test::TestRequest::get()
        .uri("/api/v1/poll")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let poll_resp = test::call_service(&app, poll_req).await;
    assert_eq!(poll_resp.status(), 200);

    let ack_req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{}/ack", review_run.id))
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .set_json(json!({}))
        .to_request();
    let ack_resp = test::call_service(&app, ack_req).await;
    assert_eq!(ack_resp.status(), 200);

    let result_req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{}/result", review_run.id))
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .set_json(json!({
            "exit_code": 0,
            "tokens_used": 1234,
            "duration_ms": 30000,
            "input_tokens": 0,
            "output_tokens": 0,
            "cache_read_tokens": 0,
            "cache_write_tokens": 0,
            "model_used": null,
            "finish_status": "completed",
            "review_url": "https://github.com/acme/app/pull/42#pullrequestreview-1",
            "review_body": "## CRITICAL\n- None\n\n## WARNINGS\n- Missing authorization check\n\n## SUGGESTIONS\n- None",
            "review_already_exists": false
        }))
        .to_request();
    let result_resp = test::call_service(&app, result_req).await;
    assert_eq!(result_resp.status(), 200);
    let result_body: serde_json::Value = test::read_body_json(result_resp).await;
    assert_eq!(result_body["status"], "completed");
    assert_eq!(
        result_body["review_url"],
        "https://github.com/acme/app/pull/42#pullrequestreview-1"
    );

    let fix_run = sqlx::query!(
        r#"SELECT status as "status: WorkRunStatus", work_type as "work_type: WorkRunType",
           review_target_pr_url, review_target_repo_full_name
           FROM work_runs WHERE parent_work_run_id = $1"#,
        review_run.id,
    )
    .fetch_one(&pool)
    .await
    .expect("fix run should be inserted");

    assert!(matches!(fix_run.status, WorkRunStatus::Pending));
    assert!(matches!(fix_run.work_type, WorkRunType::Implementation));
    assert_eq!(
        fix_run.review_target_pr_url.as_deref(),
        Some("https://github.com/acme/app/pull/42")
    );
    assert_eq!(
        fix_run.review_target_repo_full_name.as_deref(),
        Some("acme/app")
    );
}

#[sqlx::test]
async fn stale_worker_marked_disconnected(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool.clone()).await;
    let worker_id = test_helpers::insert_worker(&pool, "stale-worker").await;

    sqlx::query!(
        "UPDATE workers SET last_seen = NOW() - INTERVAL '10 minutes' WHERE id = $1",
        worker_id
    )
    .execute(&pool)
    .await
    .expect("Should update last_seen");

    let dispatcher = DispatcherService::new(
        crate::services::dispatcher::repository::DispatchRepository::new(),
        crate::services::workers::repository::WorkersRepository::new(),
        crate::services::work_runs::repository::WorkRunsRepository::new(),
        pool.clone(),
        state.dispatch_store.clone(),
        60,
        1800,
    );

    let summary = dispatcher
        .dispatch_once()
        .await
        .expect("Should run dispatch");
    assert!(
        summary.disconnected > 0,
        "Should mark at least one worker disconnected"
    );

    let row = sqlx::query!(
        r#"SELECT status as "status: WorkerStatus" FROM workers WHERE id = $1"#,
        worker_id
    )
    .fetch_one(&pool)
    .await
    .expect("Should query worker");

    assert!(matches!(row.status, WorkerStatus::Disconnected));
}
