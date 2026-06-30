use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::project_configs::model::{EffectiveProjectSettings, ProjectConfig};
use crate::models::teams::model::{DEFAULT_PROMPT_TEMPLATE, DEFAULT_REVIEW_PROMPT_TEMPLATE};
use crate::services::project_configs::service::ProjectConfigsService;

impl ProjectConfigsService {
    pub async fn effective_settings(
        &self,
        config: &ProjectConfig,
    ) -> Result<EffectiveProjectSettings, ProjectConfigsError> {
        let team = self.teams.get_team(config.team_id).await?;

        Ok(EffectiveProjectSettings {
            prompt_template: effective_prompt_template(
                config.prompt_template.as_deref(),
                &team.prompt_template,
            ),
            agents_md: config.agents_md.clone().unwrap_or(team.agents_md),
            primary_model_provider_key: config
                .primary_model_provider_key
                .clone()
                .or(team.primary_model_provider_key),
            primary_model_id: config.primary_model_id.clone().or(team.primary_model_id),
            small_model_provider_key: config
                .small_model_provider_key
                .clone()
                .or(team.small_model_provider_key),
            small_model_id: config.small_model_id.clone().or(team.small_model_id),
            review_enabled: config.review_enabled.unwrap_or(team.review_enabled),
            review_max_turns: config.review_max_turns.unwrap_or(team.review_max_turns),
            review_prompt_template: effective_review_prompt_template(
                config.review_prompt_template.as_deref(),
                &team.review_prompt_template,
            ),
            max_in_progress_tasks: config
                .max_in_progress_tasks
                .unwrap_or(team.max_in_progress_tasks),
            agent_backend: team.agent_backend.parse().unwrap_or_default(),
        })
    }
}

#[must_use]
fn effective_prompt_template(config_template: Option<&str>, team_template: &str) -> String {
    match config_template {
        Some(template) if !template.trim().is_empty() => template.to_owned(),
        Some(_) | None => prompt_template_or_default(team_template, DEFAULT_PROMPT_TEMPLATE),
    }
}

#[must_use]
fn effective_review_prompt_template(config_template: Option<&str>, team_template: &str) -> String {
    match config_template {
        Some(template) if !template.trim().is_empty() => template.to_owned(),
        Some(_) | None => prompt_template_or_default(team_template, DEFAULT_REVIEW_PROMPT_TEMPLATE),
    }
}

#[must_use]
fn prompt_template_or_default(template: &str, default_template: &str) -> String {
    if template.trim().is_empty() {
        default_template.to_owned()
    } else {
        template.to_owned()
    }
}
