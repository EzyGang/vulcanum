use std::sync::Arc;

use uuid::Uuid;

use crate::app_state::AppState;
use crate::services::auth::service::AuthService;
use crate::services::dispatcher::flag_store::InMemoryDispatchStore;
use crate::services::integration_providers::repository::IntegrationProvidersRepository;
use crate::services::integration_providers::service::IntegrationProvidersService;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::users::repository::UsersRepository;
use crate::services::users::service::UsersService;
use crate::services::work_runs::model::WorkRunStatus;
use crate::services::work_runs::repository::work_runs::InsertWorkRunParams;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::services::workers::code_store::InMemoryCodeStore;
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

pub async fn insert_project_config(pool: &sqlx::PgPool, kaneo_project_id: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs (id, kaneo_project_id, prompt_template, integration_type) VALUES ($1, $2, 'Review {{task_title}}', 'kaneo')",
        id,
        kaneo_project_id,
    )
    .execute(pool)
    .await
    .expect("Should insert project config");

    id
}

pub async fn insert_project_config_with_provider(
    pool: &sqlx::PgPool,
    kaneo_project_id: &str,
    provider_id: Uuid,
) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs (id, kaneo_project_id, prompt_template, integration_type, provider_id) VALUES ($1, $2, 'Review {{task_title}}', 'kaneo', $3)",
        id,
        kaneo_project_id,
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
    };

    repo.insert_work_run(pool, params)
        .await
        .expect("Should insert work_run")
        .id
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
        instance_password: "test-password".to_owned(),
        redis_url: String::new(),
    };

    let workers_repo = WorkersRepository::new();
    let work_runs_repo = WorkRunsRepository::new();
    let work_runs_repo_for_workers = WorkRunsRepository::new();
    let project_configs_repo = ProjectConfigsRepository::new();
    let dispatch_store = Arc::new(InMemoryDispatchStore::default());

    let auth = AuthService::new(
        UsersService::new(UsersRepository::new(), pool.clone()),
        "test-password".to_owned(),
        "test-secret".to_owned(),
    );

    AppState {
        auth,
        project_configs: ProjectConfigsService::new(
            project_configs_repo.clone(),
            pool.clone(),
            providers_repo.clone(),
        ),
        providers: providers.clone(),
        workers: WorkersService::new(
            workers_repo.clone(),
            work_runs_repo_for_workers,
            pool.clone(),
            &cfg,
            Arc::new(InMemoryCodeStore::new()),
        ),
        jobs: crate::services::work_runs::service::WorkRunsService::new(
            work_runs_repo.clone(),
            workers_repo,
            project_configs_repo,
            pool.clone(),
            dispatch_store.clone(),
            providers_repo,
            cfg.unhealthy_threshold,
        ),
        db_pool: pool,
        work_runs: work_runs_repo,
        dispatch_store,
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
