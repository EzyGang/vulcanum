use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::{EffectiveProjectSettings, ProjectConfig};
use crate::services::project_configs::service::ProjectConfigsService;

impl ProjectConfigsService {
    pub async fn effective_settings(
        &self,
        config: &ProjectConfig,
    ) -> Result<EffectiveProjectSettings, ProjectConfigsError> {
        let team = self.teams.get_team(config.team_id).await?;

        Ok(EffectiveProjectSettings {
            prompt_template: config
                .prompt_template
                .clone()
                .unwrap_or(team.prompt_template),
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
            review_pickup_column: config
                .review_pickup_column
                .clone()
                .unwrap_or(team.review_pickup_column),
            review_max_turns: config.review_max_turns.unwrap_or(team.review_max_turns),
            review_prompt_template: config
                .review_prompt_template
                .clone()
                .unwrap_or(team.review_prompt_template),
        })
    }
}
