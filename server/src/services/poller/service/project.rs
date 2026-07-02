use std::sync::Arc;

use crate::db::work_runs::queries::InsertWorkRunParams;
use crate::models::project_configs::model::ProjectConfig;
use crate::models::providers::model::IntegrationTask;
use crate::models::work_runs::model::{WorkRunStatus, WorkRunType};
use crate::services::providers::client::{IntegrationClient, TaskFetcher};

use super::{PollError, PollerService};

impl PollerService {
    pub(super) async fn poll_project(
        &self,
        config: &ProjectConfig,
    ) -> Result<(usize, usize, usize), PollError> {
        tracing::debug!(
            project_config_id = %config.id,
            team_id = %config.team_id,
            project_id = %config.external_project_id,
            provider_id = ?config.provider_id,
            pickup_column = %config.pickup_column,
            progress_column = %config.progress_column,
            target_column = %config.target_column,
            "polling project config",
        );

        let settings = match self.project_configs.effective_settings(config).await {
            Ok(settings) => settings,
            Err(e) => {
                tracing::warn!(
                    project_config_id = %config.id,
                    error = %e,
                    "skipping poll because effective settings failed to resolve"
                );
                return Ok((0, 0, 0));
            }
        };

        let remaining_capacity = self
            .remaining_capacity(config, settings.max_in_progress_tasks)
            .await;
        if remaining_capacity == 0 {
            return Ok((0, 0, 0));
        }

        let provider_id = match config.provider_id {
            Some(pid) => pid,
            None => {
                tracing::warn!(
                    project_id = %config.external_project_id,
                    "skipping poll because no provider is configured for project"
                );
                return Ok((0, 0, 0));
            }
        };
        let fetcher = match self.resolve_fetcher(config, provider_id).await {
            Some(fetcher) => fetcher,
            None => return Ok((0, 0, 0)),
        };

        let tasks = fetcher
            .fetch_tasks_in_column(&config.external_project_id, &config.pickup_column)
            .await?;
        let tasks_found = tasks.len();
        let mut inserted = 0;
        let mut skipped = 0;

        tracing::debug!(
            project_config_id = %config.id,
            project_id = %config.external_project_id,
            pickup_column = %config.pickup_column,
            tasks_found,
            remaining_capacity,
            "fetched provider tasks for project",
        );

        for task in &tasks {
            if inserted >= remaining_capacity {
                skipped += 1;
                continue;
            }

            let params = build_work_run_params(config, task);

            match self
                .work_runs_repo
                .insert_work_run_if_not_active(&self.db, params)
                .await
            {
                Ok(true) => {
                    inserted += 1;
                    self.move_task_to_progress(&fetcher, config, &task.id).await;
                }
                Ok(false) => {
                    skipped += 1;
                    tracing::debug!(
                        project_config_id = %config.id,
                        project_id = %config.external_project_id,
                        task_id = %task.id,
                        task_title = %task.title,
                        "skipped work_run insert because an active run already exists",
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to insert work_run for task {}: {}", task.id, e);
                }
            }
        }

        Ok((tasks_found, inserted, skipped))
    }

    pub(super) async fn reconcile_blocked_runs(
        &self,
        config: &ProjectConfig,
    ) -> Result<(), PollError> {
        let provider_id = match config.provider_id {
            Some(pid) => pid,
            None => return Ok(()),
        };
        let fetcher = match self.resolve_fetcher(config, provider_id).await {
            Some(fetcher) => fetcher,
            None => return Ok(()),
        };

        let blocked_runs = match self
            .work_runs_repo
            .find_blocked_by_project(&self.db, config.id)
            .await
        {
            Ok(runs) => runs,
            Err(e) => {
                tracing::warn!(
                    project_config_id = %config.id,
                    error = %e,
                    "failed to load blocked work runs for reconciliation",
                );
                return Ok(());
            }
        };

        if blocked_runs.is_empty() {
            return Ok(());
        }

        let tasks = fetcher
            .fetch_tasks_in_column(&config.external_project_id, &config.pickup_column)
            .await?;

        for run in &blocked_runs {
            if tasks.iter().any(|t| t.id == run.external_task_ref) {
                match self
                    .work_runs_repo
                    .reset_blocked_to_pending(&self.db, run.id)
                    .await
                {
                    Ok(()) => {
                        self.move_task_to_progress(&fetcher, config, &run.external_task_ref)
                            .await;
                    }
                    Err(e) => {
                        tracing::warn!(
                            work_run_id = %run.id,
                            error = %e,
                            "failed to unblock work run",
                        );
                    }
                }
            }
        }

        Ok(())
    }

    async fn remaining_capacity(&self, config: &ProjectConfig, limit: i32) -> usize {
        let active = match self
            .work_runs_repo
            .count_active_implementations_by_project(&self.db, config.id)
            .await
        {
            Ok(count) => count,
            Err(e) => {
                tracing::warn!(
                    project_config_id = %config.id,
                    error = %e,
                    "failed to count active implementation work runs",
                );
                return 0;
            }
        };
        let limit = i64::from(limit.max(1));

        limit.saturating_sub(active) as usize
    }

    async fn move_task_to_progress(
        &self,
        fetcher: &Arc<dyn TaskFetcher>,
        config: &ProjectConfig,
        task_id: &str,
    ) {
        match fetcher
            .update_task_status(task_id, &config.progress_column)
            .await
        {
            Ok(()) => (),
            Err(e) => {
                tracing::warn!(
                    project_config_id = %config.id,
                    task_id,
                    progress_column = %config.progress_column,
                    error = %e,
                    "failed to move picked up task to progress column",
                );
            }
        }
    }

    async fn resolve_fetcher(
        &self,
        config: &ProjectConfig,
        provider_id: uuid::Uuid,
    ) -> Option<Arc<dyn TaskFetcher>> {
        match &self.task_fetcher {
            Some(fetcher) => Some(Arc::clone(fetcher)),
            None => self.resolve_integration_fetcher(config, provider_id).await,
        }
    }

    async fn resolve_integration_fetcher(
        &self,
        config: &ProjectConfig,
        provider_id: uuid::Uuid,
    ) -> Option<Arc<dyn TaskFetcher>> {
        let provider = match self
            .providers_repo
            .find_by_id(&self.db, provider_id, config.team_id)
            .await
        {
            Ok(provider) => provider,
            Err(e) => {
                tracing::warn!(
                    provider_id = %provider_id,
                    project_id = %config.external_project_id,
                    error = %e,
                    "skipping poll because provider was not found"
                );
                return None;
            }
        };

        let client = IntegrationClient::from_provider(&provider);

        Some(Arc::new(client))
    }
}

fn build_work_run_params(config: &ProjectConfig, task: &IntegrationTask) -> InsertWorkRunParams {
    InsertWorkRunParams {
        team_id: config.team_id,
        external_task_ref: task.id.clone(),
        project_config_id: config.id,
        repo_full_names: config.repo_full_names.clone(),
        status: WorkRunStatus::Pending,
        work_type: WorkRunType::Implementation,
        parent_work_run_id: None,
        review_target_pr_url: None,
        review_target_repo_full_name: None,
    }
}

#[must_use]
pub(crate) fn repo_layout(repo_full_names: &[String]) -> String {
    repo_full_names
        .iter()
        .map(|name| format!("{name}: ./{}", name.replace('/', "-")))
        .collect::<Vec<String>>()
        .join("\n")
}
