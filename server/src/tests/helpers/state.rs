use super::{
    teams, AppState, Arc, AuthRepository, AuthService, GithubAppManager, GithubAppRepository,
    GithubWebhookService, GithubWebhookStore, InMemoryCancelStore, InMemoryCodeStore,
    InMemoryDeviceFlowStore, InMemoryDispatchStore, IntegrationProvidersRepository,
    IntegrationProvidersService, ModelCatalogClient, ModelProvidersRepository,
    ModelProvidersService, OpenAiChatGptDeviceAuthProvider, ProjectConfigsRepository,
    ProjectConfigsService, ProjectUsageRepository, SecretCipher, TaskAugmentationsRepository,
    TaskBoardService, TeamsRepository, TeamsService, UsersRepository, UsersService,
    WorkRunEventsRepository, WorkRunEventsService, WorkRunsRepository, WorkRunsService,
    WorkersRepository, WorkersService,
};

pub async fn build_state(pool: sqlx::PgPool) -> AppState {
    teams::ensure_default_team(&pool).await;

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
