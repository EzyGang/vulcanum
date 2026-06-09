use std::sync::Arc;

use uuid::Uuid;

use crate::app_state::AppState;
use crate::services::auth::service::AuthService;
use crate::services::dispatcher::cancel_store::InMemoryCancelStore;
use crate::services::dispatcher::dispatch_store::InMemoryDispatchStore;
use crate::services::github_app::repository::GithubAppRepository;
use crate::services::github_app::service::GithubAppManager;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::provider_configs::repository::IntegrationProvidersRepository;
use crate::services::provider_configs::service::IntegrationProvidersService;
use crate::services::users::repository::UsersRepository;
use crate::services::users::service::UsersService;
use crate::services::work_run_events::repository::WorkRunEventsRepository;
use crate::services::work_run_events::service::WorkRunEventsService;
use crate::services::work_runs::model::WorkRunStatus;
use crate::services::work_runs::repository::queries::InsertWorkRunParams;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::services::workers::registration_code_store::InMemoryCodeStore;
use crate::services::workers::repository::WorkersRepository;
use crate::services::workers::service::WorkersService;

pub async fn insert_worker(pool: &sqlx::PgPool, name: &str) -> Uuid {
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

pub async fn insert_project_config(pool: &sqlx::PgPool, external_project_id: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs (id, external_project_id, prompt_template, integration_type) VALUES ($1, $2, 'Review {{task_title}}', 'kaneo')",
        id,
        external_project_id,
    )
    .execute(pool)
    .await
    .expect("Should insert project config");

    id
}

pub async fn insert_project_config_with_provider(
    pool: &sqlx::PgPool,
    external_project_id: &str,
    provider_id: Uuid,
) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs (id, external_project_id, prompt_template, integration_type, provider_id) VALUES ($1, $2, 'Review {{task_title}}', 'kaneo', $3)",
        id,
        external_project_id,
        provider_id,
    )
    .execute(pool)
    .await
    .expect("Should insert project config");

    id
}

pub async fn insert_pending_work_run(
    pool: &sqlx::PgPool,
    project_config_id: Uuid,
    task_ref: &str,
) -> Uuid {
    let repo = WorkRunsRepository::new();
    let params = InsertWorkRunParams {
        external_task_ref: task_ref.to_owned(),
        project_config_id,
        prompt_text: "Review the PR".to_owned(),
        repo_url: String::new(),
        agents_md: String::new(),
        status: WorkRunStatus::Pending,
        task_title: None,
        task_slug: None,
    };

    repo.insert_work_run(pool, params)
        .await
        .expect("Should insert work_run")
        .id
}

pub async fn insert_running_work_run(
    pool: &sqlx::PgPool,
    project_config_id: Uuid,
    task_ref: &str,
    worker_id: Uuid,
) -> Uuid {
    let repo = WorkRunsRepository::new();
    let params = InsertWorkRunParams {
        external_task_ref: task_ref.to_owned(),
        project_config_id,
        prompt_text: "Review the PR".to_owned(),
        repo_url: String::new(),
        agents_md: String::new(),
        status: WorkRunStatus::Running,
        task_title: None,
        task_slug: None,
    };
    let id = repo
        .insert_work_run(pool, params)
        .await
        .expect("Should insert work_run")
        .id;

    sqlx::query!(
        "UPDATE work_runs SET worker_id = $1 WHERE id = $2",
        worker_id,
        id,
    )
    .execute(pool)
    .await
    .expect("Should set worker_id");

    id
}

pub fn build_state(pool: sqlx::PgPool) -> AppState {
    let providers_repo = IntegrationProvidersRepository::new();
    let providers = IntegrationProvidersService::new(providers_repo.clone(), pool.clone());

    let cfg = crate::config::AppConfig {
        db_url: String::new(),
        max_conns: 1,
        poll_period_secs: 30,
        jwt_secret: "test-secret".to_owned(),
        stale_worker_threshold_secs: 120,
        unhealthy_threshold: 3,
        stalled_running_threshold_secs: 1800,
        instance_password: "test-password".to_owned(),
        redis_url: "redis://127.0.0.1:6379".to_owned(),
        github_app_id: None,
        github_app_private_key: None,
        github_app_slug: None,
    };

    let workers_repo = WorkersRepository::new();
    let work_runs_repo = WorkRunsRepository::new();
    let work_runs_repo_for_workers = WorkRunsRepository::new();
    let project_configs_repo = ProjectConfigsRepository::new();
    let dispatch_store = Arc::new(InMemoryDispatchStore::default());
    let cancel_store = Arc::new(InMemoryCancelStore::new());
    let providers_repo_clone = providers_repo.clone();

    let github = GithubAppManager::new(
        GithubAppRepository::new(),
        pool.clone(),
        &cfg.redis_url,
        &cfg,
    )
    .expect("build github manager for tests");

    let auth = AuthService::new(
        UsersService::new(UsersRepository::new(), pool.clone()),
        "test-password".to_owned(),
        "test-secret".to_owned(),
    );

    let jobs = crate::services::work_runs::service::WorkRunsService::new(
        work_runs_repo.clone(),
        workers_repo,
        project_configs_repo.clone(),
        github.clone(),
        pool.clone(),
        dispatch_store.clone(),
        providers_repo_clone,
        cancel_store.clone(),
        cfg.unhealthy_threshold,
    );
    let events = WorkRunEventsService::new(
        WorkRunEventsRepository::new(),
        work_runs_repo.clone(),
        cancel_store.clone(),
        pool.clone(),
    );

    AppState {
        auth,
        project_configs: ProjectConfigsService::new(
            project_configs_repo,
            pool.clone(),
            crate::services::provider_configs::repository::IntegrationProvidersRepository::new(),
        ),
        providers: providers.clone(),
        workers: WorkersService::new(
            crate::services::workers::repository::WorkersRepository::new(),
            work_runs_repo_for_workers,
            pool.clone(),
            &cfg,
            Arc::new(InMemoryCodeStore::new()),
        ),
        jobs,
        events,
        github,
        db_pool: pool,
        work_runs: work_runs_repo,
        dispatch_store,
        cancel_store,
        jwt_secret: cfg.jwt_secret.clone(),
    }
}

pub fn build_worker_token(worker_id: Uuid) -> String {
    let exp = chrono::Utc::now() + chrono::Duration::minutes(15);
    let claims = serde_json::json!({"sub": worker_id.to_string(), "exp": exp.timestamp()});
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("test-secret".as_bytes()),
    )
    .expect("should build token");
    format!("Bearer {token}")
}
