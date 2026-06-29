use std::collections::HashMap;

use crate::models::project_configs::model::ProjectConfig;
use crate::models::providers::model::{CreateIntegrationLabelInput, IntegrationLabel};
use crate::models::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::services::providers::client::IntegrationClient;
use crate::services::work_runs::service::spawn_review::ReviewSpawnOutcome;
use crate::services::work_runs::service::WorkRunsService;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum LifecycleLabelState {
    ImplementationRunning,
    ReviewNeeded,
    ReviewRunning,
    NeedsAttention,
    ReadyForHuman,
}

impl LifecycleLabelState {
    const ALL: [Self; 5] = [
        Self::ImplementationRunning,
        Self::ReviewNeeded,
        Self::ReviewRunning,
        Self::NeedsAttention,
        Self::ReadyForHuman,
    ];

    const fn color(self) -> &'static str {
        match self {
            Self::ImplementationRunning => "#2563EB",
            Self::ReviewNeeded => "#D97706",
            Self::ReviewRunning => "#7C3AED",
            Self::NeedsAttention => "#DC2626",
            Self::ReadyForHuman => "#16A34A",
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::ImplementationRunning => "Implementation running",
            Self::ReviewNeeded => "Review needed",
            Self::ReviewRunning => "Review running",
            Self::NeedsAttention => "Needs attention",
            Self::ReadyForHuman => "Ready for human",
        }
    }
}

impl WorkRunsService {
    pub(crate) async fn set_lifecycle_label_for_run(
        &self,
        run: &WorkRun,
        state: LifecycleLabelState,
    ) {
        let (config, client) = match self.lifecycle_label_client(run).await {
            Some(parts) => parts,
            None => return,
        };

        let labels = match ensure_lifecycle_labels(&client, &config).await {
            Some(labels) => labels,
            None => return,
        };

        let target = match labels.get(&state) {
            Some(label) => label,
            None => {
                tracing::warn!(
                    project_config_id = %config.id,
                    state = ?state,
                    "missing lifecycle label after provisioning",
                );
                return;
            }
        };

        for other_state in LifecycleLabelState::ALL {
            if other_state == state {
                continue;
            }

            let Some(label) = labels.get(&other_state) else {
                continue;
            };

            if let Err(e) = client
                .remove_task_label(&run.external_task_ref, &label.id)
                .await
            {
                tracing::warn!(
                    work_run_id = %run.id,
                    task_ref = %run.external_task_ref,
                    label_id = %label.id,
                    label_name = %label.name,
                    error = %e,
                    "failed to remove lifecycle label",
                );
            }
        }

        if let Err(e) = client
            .add_task_label(&run.external_task_ref, &target.id)
            .await
        {
            tracing::warn!(
                work_run_id = %run.id,
                task_ref = %run.external_task_ref,
                label_id = %target.id,
                label_name = %target.name,
                error = %e,
                "failed to add lifecycle label",
            );
        }
    }

    pub(crate) async fn set_lifecycle_label_after_result(
        &self,
        run: &WorkRun,
        status: WorkRunStatus,
        review_outcome: Option<ReviewSpawnOutcome>,
    ) {
        let state = match (run.work_type, status) {
            (WorkRunType::Implementation, WorkRunStatus::Completed) => {
                match review_outcome.unwrap_or(ReviewSpawnOutcome::NoPullRequests) {
                    ReviewSpawnOutcome::NoPullRequests => Some(LifecycleLabelState::ReadyForHuman),
                    ReviewSpawnOutcome::ReviewNeeded => Some(LifecycleLabelState::ReviewNeeded),
                    ReviewSpawnOutcome::ReviewRunning => Some(LifecycleLabelState::ReviewRunning),
                }
            }
            (WorkRunType::Implementation, WorkRunStatus::Failed) => {
                Some(LifecycleLabelState::NeedsAttention)
            }
            (WorkRunType::PullRequestReview, WorkRunStatus::Completed) => {
                self.review_completion_lifecycle_label(run).await
            }
            (WorkRunType::PullRequestReview, WorkRunStatus::Failed) => {
                Some(LifecycleLabelState::NeedsAttention)
            }
            _ => None,
        };

        if let Some(state) = state {
            self.set_lifecycle_label_for_run(run, state).await;
        }
    }

    async fn review_completion_lifecycle_label(
        &self,
        run: &WorkRun,
    ) -> Option<LifecycleLabelState> {
        let parent_id = match run.parent_work_run_id {
            Some(parent_id) => parent_id,
            None => return Some(LifecycleLabelState::ReadyForHuman),
        };

        let summary = match self
            .work_runs_repo
            .review_sibling_summary(&self.db, parent_id, run.id)
            .await
        {
            Ok(summary) => summary,
            Err(e) => {
                tracing::warn!(
                    work_run_id = %run.id,
                    parent_work_run_id = %parent_id,
                    error = %e,
                    "failed to load review sibling summary for lifecycle labels",
                );
                return Some(LifecycleLabelState::ReadyForHuman);
            }
        };

        if summary.failed_count > 0 {
            return Some(LifecycleLabelState::NeedsAttention);
        }

        match summary.active_count {
            0 => Some(LifecycleLabelState::ReadyForHuman),
            _ => None,
        }
    }

    async fn lifecycle_label_client(
        &self,
        run: &WorkRun,
    ) -> Option<(ProjectConfig, IntegrationClient)> {
        let config = match self.project_configs.find_by_id(run.project_config_id).await {
            Ok(config) => config,
            Err(e) => {
                tracing::warn!(
                    work_run_id = %run.id,
                    project_config_id = %run.project_config_id,
                    error = %e,
                    "failed to load project config for lifecycle labels",
                );
                return None;
            }
        };

        if config.external_workspace_id.trim().is_empty() {
            tracing::warn!(
                work_run_id = %run.id,
                project_config_id = %config.id,
                "cannot sync lifecycle labels without provider workspace id",
            );
            return None;
        }

        let provider_id = match config.provider_id {
            Some(provider_id) => provider_id,
            None => return None,
        };

        let provider = match self
            .providers_repo
            .find_by_id(&self.db, provider_id, run.team_id)
            .await
        {
            Ok(provider) => provider,
            Err(e) => {
                tracing::warn!(
                    work_run_id = %run.id,
                    provider_id = %provider_id,
                    error = %e,
                    "failed to load provider for lifecycle labels",
                );
                return None;
            }
        };

        Some((config, IntegrationClient::from_provider(&provider)))
    }
}

async fn ensure_lifecycle_labels(
    client: &IntegrationClient,
    config: &ProjectConfig,
) -> Option<HashMap<LifecycleLabelState, IntegrationLabel>> {
    let mut provider_labels = match client.fetch_labels(&config.external_workspace_id).await {
        Ok(labels) => labels,
        Err(e) => {
            tracing::warn!(
                project_config_id = %config.id,
                workspace_id = %config.external_workspace_id,
                error = %e,
                "failed to fetch provider labels",
            );
            return None;
        }
    };
    let mut labels = HashMap::new();

    for state in LifecycleLabelState::ALL {
        match provider_labels
            .iter()
            .find(|label| label.name == state.name())
        {
            Some(label) => {
                labels.insert(state, label.clone());
            }
            None => {
                let created = match client
                    .create_label(CreateIntegrationLabelInput {
                        workspace_id: config.external_workspace_id.clone(),
                        name: state.name().to_owned(),
                        color: state.color().to_owned(),
                    })
                    .await
                {
                    Ok(label) => label,
                    Err(e) => {
                        tracing::warn!(
                            project_config_id = %config.id,
                            workspace_id = %config.external_workspace_id,
                            label_name = state.name(),
                            error = %e,
                            "failed to create provider lifecycle label",
                        );
                        return None;
                    }
                };
                provider_labels.push(created.clone());
                labels.insert(state, created);
            }
        }
    }

    Some(labels)
}
