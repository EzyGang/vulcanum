use uuid::Uuid;

use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::{ProjectConfig, UpdateProjectConfigRequest};
use crate::services::project_configs::repository::UpdateProjectConfigParams;
use crate::services::project_configs::service::{
    resolve_column_if_set, resolve_model_field, ProjectConfigsService,
};
use crate::util::github::github_repo_url;

impl ProjectConfigsService {
    pub async fn update(
        &self,
        id: Uuid,
        team_id: Uuid,
        mut params: UpdateProjectConfigRequest,
    ) -> Result<ProjectConfig, ProjectConfigsError> {
        let existing = self.repo.find_by_id(&self.db, id).await?;
        if existing.team_id != team_id {
            return Err(ProjectConfigsError::NotFound);
        }

        let provider_id = match params.provider_id {
            Some(pid) => pid,
            None => match existing.provider_id {
                Some(pid) => pid,
                None => return Err(ProjectConfigsError::NoProvider),
            },
        };

        if has_column_changes(&params) {
            let client = self.resolve_client(&provider_id, team_id).await?;
            let all_columns = client
                .fetch_columns(&existing.external_project_id)
                .await
                .map_err(ProjectConfigsError::Integration)?;

            resolve_column_if_set(&all_columns, &mut params.pickup_column)?;
            resolve_column_if_set(&all_columns, &mut params.progress_column)?;
            resolve_column_if_set(&all_columns, &mut params.target_column)?;
            resolve_nullable_column_if_set(&all_columns, &mut params.review_pickup_column)?;
        }

        let primary_provider_key = resolve_model_field(
            &params.primary_model_provider_key,
            existing.primary_model_provider_key.as_deref(),
        );
        let primary_model_id = resolve_model_field(
            &params.primary_model_id,
            existing.primary_model_id.as_deref(),
        );
        let small_provider_key = resolve_model_field(
            &params.small_model_provider_key,
            existing.small_model_provider_key.as_deref(),
        );
        let small_model_id =
            resolve_model_field(&params.small_model_id, existing.small_model_id.as_deref());

        self.validate_model_selection(team_id, primary_provider_key, primary_model_id)
            .await?;
        self.validate_model_selection(team_id, small_provider_key, small_model_id)
            .await?;

        let repo_url = params
            .repo_full_names
            .as_ref()
            .and_then(|repos| repos.first())
            .map(|full_name| github_repo_url(full_name));

        let mut tx = self
            .db
            .begin()
            .await
            .map_err(ProjectConfigsError::Database)?;
        let updated = self
            .repo
            .update(
                &mut *tx,
                id,
                &UpdateProjectConfigParams {
                    name: params.name.as_deref(),
                    pickup_column: params.pickup_column.as_deref(),
                    target_column: params.target_column.as_deref(),
                    progress_column: params.progress_column.as_deref(),
                    max_turns: params.max_turns,
                    prompt_template: params
                        .prompt_template
                        .as_ref()
                        .map(|value| value.as_deref()),
                    repo_url: repo_url.as_deref(),
                    agents_md: params.agents_md.as_ref().map(|value| value.as_deref()),
                    primary_model_provider_key: params
                        .primary_model_provider_key
                        .as_ref()
                        .map(|value| value.as_deref()),
                    primary_model_id: params
                        .primary_model_id
                        .as_ref()
                        .map(|value| value.as_deref()),
                    small_model_provider_key: params
                        .small_model_provider_key
                        .as_ref()
                        .map(|value| value.as_deref()),
                    small_model_id: params.small_model_id.as_ref().map(|value| value.as_deref()),
                    review_enabled: params.review_enabled,
                    review_pickup_column: params
                        .review_pickup_column
                        .as_ref()
                        .map(|value| value.as_deref()),
                    review_max_turns: params.review_max_turns,
                    review_prompt_template: params
                        .review_prompt_template
                        .as_ref()
                        .map(|value| value.as_deref()),
                    external_workspace_id: params.external_workspace_id.as_deref(),
                    enabled: params.enabled,
                    integration_type: params.integration_type,
                    provider_id: params.provider_id,
                },
            )
            .await?;

        match params.repo_full_names {
            Some(repo_full_names) => {
                self.repo
                    .replace_repos(&mut tx, id, &repo_full_names)
                    .await?;
                let config = self.repo.find_by_id(&mut *tx, id).await?;
                tx.commit().await.map_err(ProjectConfigsError::Database)?;
                Ok(config)
            }
            None => {
                tx.commit().await.map_err(ProjectConfigsError::Database)?;
                Ok(updated)
            }
        }
    }

    pub async fn delete(&self, id: Uuid, team_id: Uuid) -> Result<(), ProjectConfigsError> {
        let existing = self.repo.find_by_id(&self.db, id).await?;
        if existing.team_id != team_id {
            return Err(ProjectConfigsError::NotFound);
        }
        self.repo.delete(&self.db, id).await
    }
}

fn has_column_changes(params: &UpdateProjectConfigRequest) -> bool {
    params.pickup_column.is_some()
        || params.progress_column.is_some()
        || params.target_column.is_some()
        || params.review_pickup_column.is_some()
}

fn resolve_nullable_column_if_set(
    columns: &[crate::services::providers::model::IntegrationColumn],
    column: &mut Option<Option<String>>,
) -> Result<(), ProjectConfigsError> {
    match column {
        Some(Some(input)) => {
            *column = Some(Some(
                crate::services::project_configs::service::resolve_column_slug(columns, input)?,
            ));
            Ok(())
        }
        Some(None) | None => Ok(()),
    }
}
