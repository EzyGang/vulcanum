use super::{
    build_service, static_task, test_helpers, AgentBackend, SubmitResultRequest, Uuid,
    WorkRunsError, WorkRunsRepository,
};
use vulcanum_shared::api::wire::{AgentConfigPayload, JobResponse};

#[sqlx::test]
async fn submit_result_fails_if_not_running(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "early-result").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-early-1").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-early").await;

    let params = SubmitResultRequest {
        pr_urls: Vec::new(),
        exit_code: 0,
        tokens_used: 0,
        duration_ms: 0,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: None,
        result_summary: None,
        blocked_reason: None,
        review_url: None,
        review_body: None,
        review_already_exists: false,
    };
    let err = svc
        .submit_result(wr_id, worker_id, params)
        .await
        .expect_err("Should fail on pending job");

    assert!(matches!(err, WorkRunsError::InvalidStatusTransition));
}

#[sqlx::test]
async fn submit_result_fails_if_not_owner(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_a = test_helpers::workers::insert_worker(&pool, "owner-a").await;
    let worker_b = test_helpers::workers::insert_worker(&pool, "intruder-b").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-owner-1").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-owner").await;

    let dispatch_repo = crate::db::dispatcher::DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_a)
        .await
        .expect("Should dispatch");
    svc.ack_job(wr_id, worker_a)
        .await
        .expect("Worker A should ack");

    let params = SubmitResultRequest {
        pr_urls: Vec::new(),
        exit_code: 0,
        tokens_used: 0,
        duration_ms: 0,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: None,
        result_summary: None,
        blocked_reason: None,
        review_url: None,
        review_body: None,
        review_already_exists: false,
    };
    let err = svc
        .submit_result(wr_id, worker_b, params)
        .await
        .expect_err("Worker B should not submit result");

    assert!(matches!(err, WorkRunsError::NotOwned));
}

#[sqlx::test]
async fn get_job_returns_full_details(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone()).with_task_fetcher(static_task(
        "Fix login bug",
        "The login form crashes on submit.",
    ));
    let worker_id = test_helpers::workers::insert_worker(&pool, "get-job-worker").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-get-1").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-get").await;
    sqlx::query!(
        "UPDATE work_runs SET worker_id = $1 WHERE id = $2",
        worker_id,
        wr_id
    )
    .execute(&pool)
    .await
    .expect("Should assign worker");
    sqlx::query!(
        "UPDATE teams SET agent_backend = 'omp_rpc', prompt_template = $2 WHERE id = $1",
        test_helpers::DEFAULT_TEAM_ID,
        "Task {{task_title}}: {{task_body}}",
    )
    .execute(&pool)
    .await
    .expect("Should set team backend and prompt");

    let job = svc.get_job(wr_id, worker_id).await.expect("Should get job");

    assert_eq!(job.external_task_ref, "task-get");
    assert!(job
        .prompt_text
        .starts_with("Task Fix login bug: The login form crashes on submit."));
    assert!(job.prompt_text.contains("Debian-based container"));
    assert!(job.repos.is_empty());
    assert_eq!(job.agent_backend, AgentBackend::OmpRpc);
}

#[sqlx::test]
async fn get_review_job_uses_review_models_for_opencode(pool: sqlx::PgPool) {
    let job = configured_review_job(&pool, AgentBackend::OpenCode).await;

    match job.agent_config {
        AgentConfigPayload::OpenCode {
            model, small_model, ..
        } => {
            assert_eq!(model.as_deref(), Some("review-primary/review-model"));
            assert_eq!(
                small_model.as_deref(),
                Some("review-small/review-small-model")
            );
        }
        AgentConfigPayload::OmpRpc { .. } => panic!("Expected OpenCode config"),
    }
}

#[sqlx::test]
async fn get_review_job_uses_review_models_for_omp_rpc(pool: sqlx::PgPool) {
    let job = configured_review_job(&pool, AgentBackend::OmpRpc).await;

    assert_eq!(
        job.model_provider_env.get("VULCANUM_OMP_PROVIDER"),
        Some(&"review-primary".to_owned())
    );
    assert_eq!(
        job.model_provider_env.get("VULCANUM_OMP_MODEL"),
        Some(&"review-model".to_owned())
    );
    assert_eq!(
        job.model_provider_env.get("VULCANUM_OMP_SMOL"),
        Some(&"review-small/review-small-model".to_owned())
    );
}

async fn configured_review_job(pool: &sqlx::PgPool, backend: AgentBackend) -> JobResponse {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(pool, "review-model-worker").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(pool, "review-model-project").await;
    let work_run_id =
        test_helpers::work_runs::insert_pending_work_run(pool, project_id, "review-model-task")
            .await;
    let agent_backend = match backend {
        AgentBackend::OpenCode => "opencode",
        AgentBackend::OmpRpc => "omp_rpc",
    };

    sqlx::query!(
        "UPDATE work_runs SET worker_id = $1, work_type = 'pull_request_review' WHERE id = $2",
        worker_id,
        work_run_id
    )
    .execute(pool)
    .await
    .expect("Should configure review work run");
    sqlx::query!(
        r#"UPDATE teams
           SET agent_backend = $2,
               primary_model_provider_key = 'implementation-primary',
               primary_model_id = 'implementation-model',
               small_model_provider_key = 'implementation-small',
               small_model_id = 'implementation-small-model',
               review_primary_model_provider_key = 'review-primary',
               review_primary_model_id = 'review-model',
               review_small_model_provider_key = 'review-small',
               review_small_model_id = 'review-small-model'
           WHERE id = $1"#,
        test_helpers::DEFAULT_TEAM_ID,
        agent_backend,
    )
    .execute(pool)
    .await
    .expect("Should configure review model selection");

    svc.get_job(work_run_id, worker_id)
        .await
        .expect("Should get review job")
}

#[sqlx::test]
async fn job_config_fields_use_project_config_team_for_existing_config(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let stale_team_id = test_helpers::teams::insert_team(&pool, "stale-work-run-team").await;
    let project_id = test_helpers::project_configs::insert_project_config(
        &pool,
        "kaneo-project-team-for-refresh",
    )
    .await;
    let wr_id = test_helpers::work_runs::insert_pending_work_run_for_team(
        &pool,
        stale_team_id,
        project_id,
        "task-team-refresh",
    )
    .await;
    let run = WorkRunsRepository::new()
        .find_by_id(&pool, wr_id)
        .await
        .expect("Should load work run");

    let cfg = svc
        .job_config_fields_for_run(&run)
        .await
        .expect("Should load job config fields");

    assert_eq!(cfg.team_id, test_helpers::DEFAULT_TEAM_ID);
    assert_ne!(cfg.team_id, run.team_id);
}

#[sqlx::test]
async fn get_job_returns_not_found(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "missing-job-worker").await;

    let err = svc
        .get_job(Uuid::new_v4(), worker_id)
        .await
        .expect_err("Should fail");

    assert!(matches!(err, WorkRunsError::NotFound));
}

#[sqlx::test]
async fn get_job_with_repo_url_and_no_installation_fails(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "github-job-worker").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-get-2").await;

    sqlx::query!(
        "UPDATE project_configs SET repo_url = $1 WHERE id = $2",
        "https://github.com/org/repo",
        project_id
    )
    .execute(&pool)
    .await
    .expect("Should update repo_url");

    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-get-2").await;
    sqlx::query!(
        r#"INSERT INTO work_run_repos (work_run_id, repo_full_name, repo_url, position)
         VALUES ($1, $2, $3, $4)"#,
        wr_id,
        "org/repo",
        "https://github.com/org/repo",
        0,
    )
    .execute(&pool)
    .await
    .expect("Should insert work run repo snapshot");
    sqlx::query!(
        "UPDATE work_runs SET worker_id = $1 WHERE id = $2",
        worker_id,
        wr_id
    )
    .execute(&pool)
    .await
    .expect("Should assign worker");

    let err = svc
        .get_job(wr_id, worker_id)
        .await
        .expect_err("Should fail without GitHub installation");

    assert!(
        matches!(err, WorkRunsError::GithubApp(_)),
        "Expected GithubApp error, got {err:?}"
    );
}
