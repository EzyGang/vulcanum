use actix_web::{test, web, App};
use serde_json::json;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::routes;
use crate::services::work_runs::model::WorkRunStatus;
use crate::services::work_runs::repository::work_runs::InsertWorkRunParams;
use crate::services::work_runs::repository::WorkRunsRepository;

fn build_state(pool: sqlx::PgPool) -> AppState {
    let kaneo = crate::services::kaneo::client::KaneoClient::new(
        "cloud.kaneo.app".to_owned(),
        String::new(),
    );

    let cfg = crate::config::AppConfig {
        db_url: String::new(),
        max_conns: 1,
        poll_period_secs: 30,
        jwt_secret: "test-secret".to_owned(),
        stale_worker_threshold_secs: 120,
    };

    let workers_repo = crate::services::workers::repository::WorkersRepository::new();
    let work_runs_repo = WorkRunsRepository::new();
    let work_notifier = crate::services::poller::notifier::WorkNotifier::new();

    AppState {
        auth: crate::services::auth::service::AuthService::new(
            crate::services::users::service::UsersService::new(
                crate::services::users::repository::UsersRepository::new(),
                pool.clone(),
            ),
        ),
        project_configs: crate::services::project_configs::service::ProjectConfigsService::new(
            crate::services::project_configs::repository::ProjectConfigsRepository::new(),
            pool.clone(),
            kaneo.clone(),
        ),
        workers: crate::services::workers::service::WorkersService::new(
            workers_repo.clone(),
            pool.clone(),
            &cfg,
        ),
        jobs: crate::services::work_runs::service::WorkRunsService::new(
            work_runs_repo.clone(),
            workers_repo,
            pool.clone(),
            work_notifier.clone(),
            120,
        ),
        db_pool: pool,
        kaneo,
        work_runs: work_runs_repo,
        work_notifier,
    }
}

async fn insert_worker(pool: &sqlx::PgPool, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    let hash = hex::encode([0u8; 32]);

    sqlx::query!(
        "INSERT INTO workers (id, name, refresh_token_hash, refresh_expires_at, status) VALUES ($1, $2, $3, NOW() + INTERVAL '30 days', 'idle'::worker_status)",
        id,
        name,
        hash,
    )
    .execute(pool)
    .await
    .expect("Should insert worker");

    id
}

async fn insert_project_config(pool: &sqlx::PgPool, kaneo_project_id: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs (id, kaneo_project_id, prompt_template) VALUES ($1, $2, 'Review {{task_title}}')",
        id,
        kaneo_project_id,
    )
    .execute(pool)
    .await
    .expect("Should insert project config");

    id
}

async fn insert_pending_work_run(
    pool: &sqlx::PgPool,
    project_config_id: Uuid,
    task_ref: &str,
) -> Uuid {
    let repo = WorkRunsRepository::new();
    let params = InsertWorkRunParams {
        external_task_ref: task_ref.to_owned(),
        project_config_id,
        prompt_text: "Review the PR".to_owned(),
        status: WorkRunStatus::Pending,
    };

    repo.insert_work_run(pool, params)
        .await
        .expect("Should insert work_run")
        .id
}

#[sqlx::test]
async fn poll_returns_204_when_no_work(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let worker_id = insert_worker(&pool, "test-poll-noop").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/poll?worker_id={worker_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 204);
}

#[sqlx::test]
async fn poll_returns_job_id_when_work_available(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let worker_id = insert_worker(&pool, "test-poll-work").await;
    let project_id = insert_project_config(&pool, "kaneo-poll-test").await;
    let wr_id = insert_pending_work_run(&pool, project_id, "task-poll-test").await;

    state.work_notifier.add_worker(worker_id).await;
    state.work_notifier.notify_all().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/poll?worker_id={worker_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["job_id"], wr_id.to_string());
}

#[sqlx::test]
async fn get_job_returns_200(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let project_id = insert_project_config(&pool, "kaneo-get-test").await;
    let wr_id = insert_pending_work_run(&pool, project_id, "task-get-test").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/jobs/{wr_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], wr_id.to_string());
    assert_eq!(body["external_task_ref"], "task-get-test");
}

#[sqlx::test]
async fn get_job_returns_404_for_missing(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/jobs/{}", Uuid::new_v4()))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 404);
}

#[sqlx::test]
async fn ack_job_returns_200(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let worker_id = insert_worker(&pool, "test-acker").await;
    let project_id = insert_project_config(&pool, "kaneo-ack-test").await;
    let wr_id = insert_pending_work_run(&pool, project_id, "task-ack-test").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/ack"))
        .set_json(json!({"worker_id": worker_id}))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "running");
    assert_eq!(body["worker_id"], worker_id.to_string());
}

#[sqlx::test]
async fn ack_job_returns_409_when_already_claimed(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let worker_a = insert_worker(&pool, "test-acker-a").await;
    let worker_b = insert_worker(&pool, "test-acker-b").await;
    let project_id = insert_project_config(&pool, "kaneo-ack-race").await;
    let wr_id = insert_pending_work_run(&pool, project_id, "task-ack-race").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req_a = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/ack"))
        .set_json(json!({"worker_id": worker_a}))
        .to_request();
    let resp_a = test::call_service(&app, req_a).await;
    assert_eq!(resp_a.status(), 200);

    let req_b = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/ack"))
        .set_json(json!({"worker_id": worker_b}))
        .to_request();
    let resp_b = test::call_service(&app, req_b).await;
    assert_eq!(resp_b.status(), 409);
}

#[sqlx::test]
async fn submit_result_returns_200_on_completed(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let worker_id = insert_worker(&pool, "test-result").await;
    let project_id = insert_project_config(&pool, "kaneo-result-test").await;
    let wr_id = insert_pending_work_run(&pool, project_id, "task-result-test").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let ack_req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/ack"))
        .set_json(json!({"worker_id": worker_id}))
        .to_request();
    test::call_service(&app, ack_req).await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/result"))
        .set_json(json!({
            "pr_url": "https://github.com/test/pr/1",
            "exit_code": 0,
            "tokens_used": 1000,
            "duration_ms": 60000,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "completed");
    assert_eq!(body["result_pr_url"], "https://github.com/test/pr/1");
    assert_eq!(body["result_exit_code"], 0);
    assert_eq!(body["tokens_used"], 1000);
    assert_eq!(body["duration_ms"], 60000);
}

#[sqlx::test]
async fn submit_result_returns_409_when_not_running(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let project_id = insert_project_config(&pool, "kaneo-early-result").await;
    let wr_id = insert_pending_work_run(&pool, project_id, "task-early-result").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/result"))
        .set_json(json!({
            "pr_url": "",
            "exit_code": 0,
            "tokens_used": 0,
            "duration_ms": 0,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 409);
}
