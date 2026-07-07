use std::path::Path;
use std::sync::Arc;

use tokio::sync::{watch, RwLock};
use uuid::Uuid;

use vulcanum_shared::api_types::JobResponse;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::config::WorkerConfig;
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};
use vulcanum_shared::worker_state::WorkerState;

use crate::daemon::job::execution::event_reporter::EventReporter;
use crate::daemon::job::execution::submit::{submit_failed_result, FailedResult};
use crate::daemon::job::github_credentials::{spawn_refresh_task, stop_refresh_task};
use crate::daemon::job::runtime_secrets::job_runtime_secrets;
use crate::isolation::factory::{create_isolation_provider, IsolationKind};
use crate::state::journal::Journal;

pub(super) struct PreparedEnvironment {
    pub provider: IsolationKind,
    pub isolated_env: IsolatedEnvironment,
    pub github_refresh_stop: Option<watch::Sender<bool>>,
}

pub(super) struct PrepareEnvironmentCtx<'a> {
    pub client: Arc<ApiClient>,
    pub worker_state: Arc<RwLock<WorkerState>>,
    pub journal: Arc<Journal>,
    pub reporter: Arc<EventReporter>,
    pub worker_id: Uuid,
    pub job_id: Uuid,
    pub job: &'a JobResponse,
    pub config: &'a WorkerConfig,
    pub workdir: &'a Path,
}

pub(super) async fn prepare_environment(
    ctx: PrepareEnvironmentCtx<'_>,
) -> Result<PreparedEnvironment, ()> {
    let PrepareEnvironmentCtx {
        client,
        worker_state,
        journal,
        reporter,
        worker_id,
        job_id,
        job,
        config,
        workdir,
    } = ctx;
    if let Err(e) = reset_fresh_workdir(workdir).await {
        tracing::error!(work_run_id = %job_id, workdir = %workdir.display(), error = %e, "failed to reset stale workdir");
        submit_failed_result(
            client,
            worker_state,
            journal,
            job_id,
            &FailedResult::empty(),
        )
        .await;
        return Err(());
    }

    if let Err(e) = tokio::fs::create_dir_all(workdir).await {
        tracing::error!(work_run_id = %job_id, error = %e, "failed to create workdir");
        submit_failed_result(
            client,
            worker_state,
            journal,
            job_id,
            &FailedResult::empty(),
        )
        .await;
        return Err(());
    }

    let provider = match create_isolation_provider(config) {
        Ok(provider) => provider,
        Err(e) => {
            tracing::error!(
                worker_id = %worker_id,
                work_run_id = %job_id,
                external_task_ref = %job.external_task_ref,
                error = %e,
                "isolation provider selection failed",
            );
            reporter
                .emit(
                    "session.failed",
                    serde_json::json!({"reason": "isolation_provider_selection_failed"}),
                )
                .await;
            reporter.shutdown().await;
            submit_failed_result(
                client,
                worker_state,
                journal,
                job_id,
                &FailedResult::empty(),
            )
            .await;
            return Err(());
        }
    };

    let limits = ResourceLimits::default();
    let secrets = job_runtime_secrets(job);
    let env_vars = std::collections::HashMap::new();
    let isolated_env = match provider
        .prepare(
            workdir,
            &secrets,
            &env_vars,
            &limits,
            job.work_type,
            &job.agents_md,
            job.agent_backend,
            &job.agent_config,
            &job.repos,
        )
        .await
    {
        Ok(env) => env,
        Err(e) => {
            tracing::error!(
                worker_id = %worker_id,
                work_run_id = %job_id,
                external_task_ref = %job.external_task_ref,
                error = %e,
                "isolation prepare failed",
            );
            reporter
                .emit(
                    "session.failed",
                    serde_json::json!({"reason": "isolation_prepare_failed"}),
                )
                .await;
            reporter.shutdown().await;
            submit_failed_result(
                client,
                worker_state,
                journal,
                job_id,
                &FailedResult::empty(),
            )
            .await;
            return Err(());
        }
    };

    let github_refresh_stop = job.github_token.as_ref().map(|_| {
        spawn_refresh_task(
            client.clone(),
            worker_state.clone(),
            job_id,
            workdir.to_path_buf(),
            job.github_token_expires_at,
        )
    });

    if let (Some(pr_url), Some(repo_full_name)) = (
        job.review_target_pr_url.as_deref(),
        job.review_target_repo_full_name.as_deref(),
    ) {
        match crate::isolation::checkout::checkout_pull_request(
            &isolated_env.workspace_dir,
            &isolated_env.repos,
            repo_full_name,
            pr_url,
            &crate::isolation::github_credentials::host_command_env(&isolated_env.workdir),
        )
        .await
        {
            Ok(()) => (),
            Err(e) => {
                tracing::error!(
                    worker_id = %worker_id,
                    work_run_id = %job_id,
                    repo = %repo_full_name,
                    pr_url = %pr_url,
                    error = %e,
                    "pull request checkout failed",
                );
                stop_refresh_task(github_refresh_stop);
                provider.cleanup(&isolated_env).await;
                submit_failed_result(
                    client,
                    worker_state,
                    journal,
                    job_id,
                    &FailedResult::empty(),
                )
                .await;
                return Err(());
            }
        }
    }

    Ok(PreparedEnvironment {
        provider,
        isolated_env,
        github_refresh_stop,
    })
}

async fn reset_fresh_workdir(workdir: &Path) -> std::io::Result<()> {
    if !is_safe_workdir(workdir) || !tokio::fs::try_exists(workdir).await? {
        return Ok(());
    }

    tokio::fs::remove_dir_all(workdir).await
}

fn is_safe_workdir(path: &Path) -> bool {
    let name = path.file_name().and_then(|name| name.to_str());
    match name {
        Some(name) if name.starts_with("vulcanum-work-") => {
            path.parent() == Some(std::env::temp_dir().as_path())
        }
        _ => false,
    }
}
