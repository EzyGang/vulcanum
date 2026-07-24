use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;

use uuid::Uuid;

use crate::app_state::AppState;
use crate::db::auth::AuthRepository;
use crate::db::github_app::GithubAppRepository;
use crate::db::model_providers::ModelProvidersRepository;
use crate::db::project_configs::ProjectConfigsRepository;
use crate::db::project_usage::ProjectUsageRepository;
use crate::db::provider_configs::IntegrationProvidersRepository;
use crate::db::task_augmentations::TaskAugmentationsRepository;
use crate::db::teams::TeamsRepository;
use crate::db::users::UsersRepository;
use crate::db::work_run_events::WorkRunEventsRepository;
use crate::db::work_runs::queries::InsertWorkRunParams;
use crate::db::work_runs::WorkRunsRepository;
use crate::db::workers::WorkersRepository;
use crate::models::teams::model::{DEFAULT_PROMPT_TEMPLATE, DEFAULT_REVIEW_PROMPT_TEMPLATE};
use crate::models::work_runs::model::{WorkRunStatus, WorkRunType};
use crate::services::auth::service::AuthService;
use crate::services::dispatcher::cancel_store::InMemoryCancelStore;
use crate::services::dispatcher::dispatch_store::InMemoryDispatchStore;
use crate::services::github_app::service::webhooks::GithubWebhookService;
use crate::services::github_app::service::GithubAppManager;
use crate::services::github_app::webhook_store::GithubWebhookStore;
use crate::services::model_providers::auth::device_flow::InMemoryDeviceFlowStore;
use crate::services::model_providers::auth::encryption::SecretCipher;
use crate::services::model_providers::auth::openai_chatgpt::OpenAiChatGptDeviceAuthProvider;
use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::service::ModelProvidersService;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::provider_configs::service::IntegrationProvidersService;
use crate::services::task_board::service::TaskBoardService;
use crate::services::teams::service::TeamsService;
use crate::services::users::service::UsersService;
use crate::services::work_run_events::service::WorkRunEventsService;
use crate::services::work_runs::service::WorkRunsService;
use crate::services::workers::registration_code_store::InMemoryCodeStore;
use crate::services::workers::service::WorkersService;

pub const DEFAULT_TEAM_ID: Uuid = Uuid::from_u128(1);
pub const GITHUB_WEBHOOK_SECRET: &str = "webhook-test-secret";

pub fn github_webhook_payload(action: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "action": action,
        "number": 42,
        "installation": {"id": 123},
        "repository": {"full_name": "acme/widgets"},
    }))
    .expect("serialize GitHub webhook fixture")
}

pub fn sign_github_webhook(body: &[u8]) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(GITHUB_WEBHOOK_SECRET.as_bytes()).expect("valid HMAC key");
    mac.update(body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

pub async fn ensure_default_team(pool: &sqlx::PgPool) {
    sqlx::query!(
        r#"INSERT INTO teams (id, name, prompt_template, review_prompt_template)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (id) DO UPDATE
           SET name = EXCLUDED.name,
               prompt_template = EXCLUDED.prompt_template,
               review_prompt_template = EXCLUDED.review_prompt_template"#,
        DEFAULT_TEAM_ID,
        "Default team",
        "",
        "",
    )
    .execute(pool)
    .await
    .expect("Should ensure default team");
}

pub async fn insert_team(pool: &sqlx::PgPool, name: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        r#"INSERT INTO teams (id, name, prompt_template, review_prompt_template)
           VALUES ($1, $2, $3, $4)"#,
        id,
        name,
        DEFAULT_PROMPT_TEMPLATE,
        DEFAULT_REVIEW_PROMPT_TEMPLATE,
    )
    .execute(pool)
    .await
    .expect("Should insert team");

    id
}

pub async fn insert_user(pool: &sqlx::PgPool, id: &str) {
    let email = format!("{id}@example.com");

    sqlx::query!("INSERT INTO users (id, email) VALUES ($1, $2)", id, email)
        .execute(pool)
        .await
        .expect("Should insert user");
}

pub async fn insert_worker(pool: &sqlx::PgPool, name: &str) -> Uuid {
    ensure_default_team(pool).await;
    insert_worker_for_team(pool, DEFAULT_TEAM_ID, name).await
}

pub async fn insert_worker_for_team(pool: &sqlx::PgPool, team_id: Uuid, name: &str) -> Uuid {
    if team_id == DEFAULT_TEAM_ID {
        ensure_default_team(pool).await;
    }

    let id = Uuid::new_v4();
    let hash = hex::encode([0u8; 32]);

    sqlx::query!(
        "INSERT INTO workers (id, team_id, name, refresh_token_hash, refresh_expires_at, status) VALUES ($1, $2, $3, $4, NOW() + INTERVAL '30 days', 'idle'::worker_status)",
        id,
        team_id,
        name,
        hash,
    )
    .execute(pool)
    .await
    .expect("Should insert worker");

    id
}

pub async fn insert_project_config(pool: &sqlx::PgPool, external_project_id: &str) -> Uuid {
    ensure_default_team(pool).await;
    insert_project_config_for_team(pool, DEFAULT_TEAM_ID, external_project_id).await
}

pub async fn insert_project_config_for_team(
    pool: &sqlx::PgPool,
    team_id: Uuid,
    external_project_id: &str,
) -> Uuid {
    if team_id == DEFAULT_TEAM_ID {
        ensure_default_team(pool).await;
    }

    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs (id, team_id, external_project_id, integration_type) VALUES ($1, $2, $3, 'kaneo')",
        id,
        team_id,
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
    ensure_default_team(pool).await;
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs (id, team_id, external_project_id, integration_type, provider_id) VALUES ($1, $2, $3, 'kaneo', $4)",
        id,
        DEFAULT_TEAM_ID,
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
    ensure_default_team(pool).await;
    insert_pending_work_run_for_team(pool, DEFAULT_TEAM_ID, project_config_id, task_ref).await
}

pub async fn insert_pending_work_run_for_team(
    pool: &sqlx::PgPool,
    team_id: Uuid,
    project_config_id: Uuid,
    task_ref: &str,
) -> Uuid {
    if team_id == DEFAULT_TEAM_ID {
        ensure_default_team(pool).await;
    }

    let repo = WorkRunsRepository::new();
    let params = InsertWorkRunParams {
        team_id,
        external_task_ref: task_ref.to_owned(),
        task_title: None,
        task_slug: None,
        project_config_id,
        repo_full_names: Vec::new(),
        status: WorkRunStatus::Pending,
        work_type: WorkRunType::Implementation,
        parent_work_run_id: None,
        review_target_pr_url: None,
        review_target_repo_full_name: None,
        github_installation_id: None,
        github_delivery_id: None,
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
    ensure_default_team(pool).await;
    insert_running_work_run_for_team(
        pool,
        DEFAULT_TEAM_ID,
        project_config_id,
        task_ref,
        worker_id,
    )
    .await
}

pub async fn insert_running_work_run_for_team(
    pool: &sqlx::PgPool,
    team_id: Uuid,
    project_config_id: Uuid,
    task_ref: &str,
    worker_id: Uuid,
) -> Uuid {
    if team_id == DEFAULT_TEAM_ID {
        ensure_default_team(pool).await;
    }

    let repo = WorkRunsRepository::new();
    let params = InsertWorkRunParams {
        team_id,
        external_task_ref: task_ref.to_owned(),
        task_title: None,
        task_slug: None,
        project_config_id,
        repo_full_names: Vec::new(),
        status: WorkRunStatus::Running,
        work_type: WorkRunType::Implementation,
        parent_work_run_id: None,
        review_target_pr_url: None,
        review_target_repo_full_name: None,
        github_installation_id: None,
        github_delivery_id: None,
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

    sqlx::query!(
        "UPDATE workers SET active_jobs = active_jobs + 1, status = 'busy'::worker_status WHERE id = $1",
        worker_id,
    )
    .execute(pool)
    .await
    .expect("Should reserve worker capacity");

    id
}

pub async fn build_state(pool: sqlx::PgPool) -> AppState {
    ensure_default_team(&pool).await;

    let providers_repo = IntegrationProvidersRepository::new();
    let providers = IntegrationProvidersService::new(providers_repo.clone(), pool.clone());
    let model_catalog = ModelCatalogClient::new().expect("build model catalog client");
    let model_providers_repo = ModelProvidersRepository::new();
    let model_providers = ModelProvidersService::new(
        model_providers_repo.clone(),
        pool.clone(),
        model_catalog.clone(),
        SecretCipher::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").expect("test cipher"),
        Arc::new(InMemoryDeviceFlowStore::new()),
        Arc::new(OpenAiChatGptDeviceAuthProvider::new().expect("build device auth client")),
    );

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
        model_provider_secret_key: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_owned(),
        is_single_user: true,
        github_app_id: None,
        github_app_private_key: None,
        github_app_slug: None,
        github_webhook_secret: None,
        github_oauth_client_id: None,
        github_oauth_client_secret: None,
        github_oauth_redirect_url: None,
    };

    let workers_repo = WorkersRepository::new();
    let work_runs_repo = WorkRunsRepository::new();
    let work_runs_repo_for_workers = WorkRunsRepository::new();
    let project_configs_repo = ProjectConfigsRepository::new();
    let project_usage_repo = ProjectUsageRepository::new();
    let task_board = TaskBoardService::new(
        pool.clone(),
        providers_repo.clone(),
        project_configs_repo.clone(),
        TaskAugmentationsRepository::new(),
        project_usage_repo.clone(),
    );
    let dispatch_store = Arc::new(InMemoryDispatchStore::default());
    let cancel_store = Arc::new(InMemoryCancelStore::new());
    let providers_repo_clone = providers_repo.clone();
    let teams = TeamsService::new(TeamsRepository::new(), pool.clone());
    let project_configs = ProjectConfigsService::new(
        project_configs_repo.clone(),
        pool.clone(),
        providers_repo.clone(),
        teams.clone(),
    );

    let github = GithubAppManager::new(
        GithubAppRepository::new(),
        pool.clone(),
        &cfg.redis_url,
        &cfg,
    )
    .expect("build github manager for tests");

    let auth = AuthService::new(
        AuthRepository::new(),
        GithubAppRepository::new(),
        pool.clone(),
        UsersService::new(UsersRepository::new(), pool.clone()),
        teams.clone(),
        &cfg,
    )
    .expect("build auth service");

    let jobs = WorkRunsService::new(
        work_runs_repo.clone(),
        TaskAugmentationsRepository::new(),
        project_usage_repo,
        workers_repo,
        project_configs.clone(),
        github.clone(),
        pool.clone(),
        dispatch_store.clone(),
        providers_repo_clone,
        model_providers.clone(),
        cancel_store.clone(),
        cfg.unhealthy_threshold,
    );
    let github_webhooks = GithubWebhookService::new(
        cfg.github_webhook_secret.as_deref().map(Arc::<str>::from),
        cfg.github_app_slug.as_deref().map(Arc::<str>::from),
        cfg.is_single_user,
        GithubWebhookStore::in_memory(),
        jobs.clone(),
        Arc::new(github.clone()),
    );
    let events = WorkRunEventsService::new(
        WorkRunEventsRepository::new(),
        work_runs_repo.clone(),
        cancel_store.clone(),
        pool.clone(),
    );

    AppState {
        auth,
        project_configs,
        providers: providers.clone(),
        task_board,
        model_providers,
        workers: WorkersService::new(
            WorkersRepository::new(),
            work_runs_repo_for_workers,
            pool.clone(),
            &cfg,
            Arc::new(InMemoryCodeStore::new()),
        ),
        jobs,
        events,
        github,
        github_webhooks,
        teams,
        jwt_secret: cfg.jwt_secret.clone(),
        is_single_user: cfg.is_single_user,
    }
}

pub fn build_worker_token(worker_id: Uuid) -> String {
    let exp = chrono::Utc::now() + chrono::Duration::minutes(15);
    let claims =
        serde_json::json!({"sub": worker_id.to_string(), "typ": "worker", "exp": exp.timestamp()});
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("test-secret".as_bytes()),
    )
    .expect("should build token");
    format!("Bearer {token}")
}
