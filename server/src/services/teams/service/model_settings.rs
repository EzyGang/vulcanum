use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::team_auth::TeamPrincipal;
use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::repository::ModelProvidersRepository;
use crate::services::model_providers::service::ModelProvidersService;
use crate::services::teams::errors::TeamsError;
use crate::services::teams::model::{Team, UpdateTeamRequest};
use crate::services::teams::service::{validate_team_name, TeamsService};

impl TeamsService {
    pub async fn update_for_principal(
        &self,
        team_id: Uuid,
        params: &UpdateTeamRequest,
        principal: &TeamPrincipal,
        single_user: bool,
    ) -> Result<Team, TeamsError> {
        let name = match params.name.as_deref() {
            Some(name) => Some(validate_team_name(name)?),
            None => None,
        };
        self.authorize_owner(team_id, principal, single_user)
            .await?;
        let current = self.repo.get_by_id(&self.db, team_id).await?;
        self.validate_model_selection(
            team_id,
            resolve_model_provider_config_id(
                params.primary_model_provider_config_id,
                current.primary_model_provider_config_id,
            ),
            resolve_model_id(
                &params.primary_model_id,
                current.primary_model_id.as_deref(),
            ),
        )
        .await?;
        self.validate_model_selection(
            team_id,
            resolve_model_provider_config_id(
                params.small_model_provider_config_id,
                current.small_model_provider_config_id,
            ),
            resolve_model_id(&params.small_model_id, current.small_model_id.as_deref()),
        )
        .await?;
        self.repo
            .update_settings(
                &self.db,
                team_id,
                name,
                params.prompt_template.as_deref(),
                params.agents_md.as_deref(),
                params
                    .primary_model_provider_key
                    .as_ref()
                    .map(|value| value.as_deref()),
                params.primary_model_provider_config_id,
                params
                    .primary_model_id
                    .as_ref()
                    .map(|value| value.as_deref()),
                params
                    .small_model_provider_key
                    .as_ref()
                    .map(|value| value.as_deref()),
                params.small_model_provider_config_id,
                params.small_model_id.as_ref().map(|value| value.as_deref()),
                params.review_enabled,
                params.review_pickup_column.as_deref(),
                params.review_max_turns,
                params.review_prompt_template.as_deref(),
                params.max_in_progress_tasks,
            )
            .await
    }

    async fn validate_model_selection(
        &self,
        team_id: Uuid,
        provider_config_id: Option<Uuid>,
        model_id: Option<&str>,
    ) -> Result<(), TeamsError> {
        self.model_providers
            .validate_model_selection(team_id, provider_config_id, model_id)
            .await
            .map_err(TeamsError::ModelProvider)
    }
}

pub(super) fn default_model_providers(db: PgPool) -> ModelProvidersService {
    ModelProvidersService::new(
        ModelProvidersRepository::new(),
        db,
        ModelCatalogClient::new(),
        "team-model-provider-validation",
    )
}

fn resolve_model_provider_config_id(
    field: Option<Option<Uuid>>,
    existing: Option<Uuid>,
) -> Option<Uuid> {
    match field {
        Some(value) => value,
        None => existing,
    }
}

fn resolve_model_id<'a>(
    field: &'a Option<Option<String>>,
    existing: Option<&'a str>,
) -> Option<&'a str> {
    match field {
        Some(value) => value.as_deref(),
        None => existing,
    }
}
