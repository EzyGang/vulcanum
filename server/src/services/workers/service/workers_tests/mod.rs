mod connection;
mod management;
mod refresh;

use std::sync::Arc;

use crate::config::AppConfig;
use crate::db::workers::queries::CreateWorkerParams;
use crate::db::workers::WorkersRepository;
use crate::models::workers::errors::WorkersError;
use crate::models::workers::model::DEFAULT_MAX_CONCURRENT_JOBS;
use crate::services::workers::registration_code_store::InMemoryCodeStore;
use crate::services::workers::service::WorkersService;
use crate::test_helpers::DEFAULT_TEAM_ID;
use chrono::{Duration, Utc};
use vulcanum_shared::api::wire::{ConnectRequest, RefreshRequest, WorkerCapabilities};

fn cfg() -> AppConfig {
    AppConfig {
        db_url: String::new(),
        max_conns: 1,
        poll_period_secs: 30,
        jwt_secret: "test-secret".to_owned(),
        stale_worker_threshold_secs: 120,
        unhealthy_threshold: 3,
        stalled_running_threshold_secs: 1800,
        instance_password: "test-password".to_owned(),
        is_single_user: true,
        redis_url: "redis://127.0.0.1:6379".to_owned(),
        model_provider_secret_key: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_owned(),
        github_app_id: None,
        github_app_private_key: None,
        github_app_slug: None,
        github_webhook_secret: None,
        github_oauth_client_id: None,
        github_oauth_client_secret: None,
        github_oauth_redirect_url: None,
    }
}

async fn svc(pool: sqlx::PgPool) -> WorkersService {
    crate::test_helpers::teams::ensure_default_team(&pool).await;

    let c = cfg();
    WorkersService::new(
        WorkersRepository::new(),
        crate::db::work_runs::WorkRunsRepository::new(),
        pool,
        &c,
        Arc::new(InMemoryCodeStore::new()),
    )
}
