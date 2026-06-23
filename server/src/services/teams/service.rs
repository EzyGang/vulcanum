pub mod invites;

use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::team_auth::TeamPrincipal;
use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::repository::ModelProvidersRepository;
use crate::services::model_providers::service::ModelProvidersService;
use crate::services::teams::errors::TeamsError;
use crate::services::teams::invite_store::{InMemoryTeamInviteStore, TeamInviteStore};
use crate::services::teams::model::{ProviderIdentity, Team, TeamMemberInfo, UpdateTeamRequest};
use crate::services::teams::repository::TeamsRepository;

#[derive(Clone)]
pub struct TeamsService {
    pub repo: TeamsRepository,
    pub db: PgPool,
    pub invite_store: Arc<dyn TeamInviteStore>,
    model_providers: ModelProvidersService,
}

impl TeamsService {
    pub fn new(repo: TeamsRepository, db: PgPool) -> Self {
        Self::new_with_invite_store(repo, db, Arc::new(InMemoryTeamInviteStore::new()))
    }

    pub fn new_with_invite_store(
        repo: TeamsRepository,
        db: PgPool,
        invite_store: Arc<dyn TeamInviteStore>,
    ) -> Self {
        let model_providers = default_model_providers(db.clone());
        Self::new_with_model_providers(repo, db, invite_store, model_providers)
    }

    pub fn new_with_model_providers(
        repo: TeamsRepository,
        db: PgPool,
        invite_store: Arc<dyn TeamInviteStore>,
        model_providers: ModelProvidersService,
    ) -> Self {
        Self {
            repo,
            db,
            invite_store,
            model_providers,
        }
    }

    #[must_use = "team list results should be handled"]
    pub async fn list_for_user(&self, user_id: &str) -> Result<Vec<Team>, TeamsError> {
        self.repo.list_for_user(&self.db, user_id).await
    }

    #[must_use = "team list results should be handled"]
    pub async fn list_all(&self) -> Result<Vec<Team>, TeamsError> {
        self.repo.list_all(&self.db).await
    }

    #[must_use = "team list results should be handled"]
    pub async fn list_for_principal(
        &self,
        principal: &TeamPrincipal,
        single_user: bool,
    ) -> Result<Vec<Team>, TeamsError> {
        match principal {
            TeamPrincipal::Instance { .. } => match single_user {
                true => self.list_all().await,
                false => Err(TeamsError::AccessDenied),
            },
            TeamPrincipal::User { user_id, .. } => self.list_for_user(user_id).await,
        }
    }

    #[must_use = "team lookup results should be handled"]
    pub async fn get_team(&self, team_id: Uuid) -> Result<Team, TeamsError> {
        self.repo.get_by_id(&self.db, team_id).await
    }

    #[must_use = "team lookup results should be handled"]
    pub async fn get_for_principal(
        &self,
        team_id: Uuid,
        principal: &TeamPrincipal,
        single_user: bool,
    ) -> Result<Team, TeamsError> {
        self.authorize_team_read(team_id, principal, single_user)
            .await?;
        self.repo.get_by_id(&self.db, team_id).await
    }

    pub async fn create_for_principal(
        &self,
        name: &str,
        principal: &TeamPrincipal,
        single_user: bool,
    ) -> Result<Team, TeamsError> {
        let name = validate_team_name(name)?;
        let mut tx = self.db.begin().await?;
        let team = self.repo.create_team(&mut *tx, name).await?;

        match principal {
            TeamPrincipal::Instance { .. } => {
                if !single_user {
                    return Err(TeamsError::AccessDenied);
                }
            }
            TeamPrincipal::User { user_id, .. } => {
                self.repo
                    .add_member(&mut *tx, team.id, user_id, "owner")
                    .await?;
            }
        }

        tx.commit().await?;
        Ok(team)
    }

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

    pub async fn delete_for_principal(
        &self,
        team_id: Uuid,
        principal: &TeamPrincipal,
        single_user: bool,
    ) -> Result<(), TeamsError> {
        self.authorize_owner(team_id, principal, single_user)
            .await?;
        let team = self.repo.get_by_id(&self.db, team_id).await?;
        if team.personal_user_id.is_some() {
            return Err(TeamsError::InvalidOperation(
                "Personal teams cannot be deleted".to_owned(),
            ));
        }

        match principal {
            TeamPrincipal::Instance { .. } => {
                let count = self.repo.count_all(&self.db).await?;
                if count <= 1 {
                    return Err(TeamsError::InvalidOperation(
                        "Cannot delete the last team".to_owned(),
                    ));
                }
            }
            TeamPrincipal::User { user_id, .. } => {
                let count = self.repo.count_for_user(&self.db, user_id).await?;
                if count <= 1 {
                    return Err(TeamsError::InvalidOperation(
                        "Cannot delete your last team".to_owned(),
                    ));
                }
            }
        }

        self.repo.delete(&self.db, team_id).await
    }

    #[must_use = "member list results should be handled"]
    pub async fn list_members_for_principal(
        &self,
        team_id: Uuid,
        principal: &TeamPrincipal,
        single_user: bool,
    ) -> Result<Vec<TeamMemberInfo>, TeamsError> {
        self.authorize_team_read(team_id, principal, single_user)
            .await?;
        self.repo.list_members(&self.db, team_id).await
    }

    #[must_use = "identity list results should be handled"]
    pub async fn list_identities_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<ProviderIdentity>, TeamsError> {
        self.repo.list_identities_for_user(&self.db, user_id).await
    }

    #[must_use = "resolved team id should be used"]
    pub async fn resolve_team(
        &self,
        principal: &TeamPrincipal,
        single_user: bool,
    ) -> Result<Uuid, TeamsError> {
        match principal {
            TeamPrincipal::Instance { team_id } => {
                if !single_user {
                    return Err(TeamsError::AccessDenied);
                }
                match team_id {
                    Some(team_id) => Ok(self.repo.get_by_id(&self.db, *team_id).await?.id),
                    None => Ok(self.repo.get_default_team(&self.db).await?.id),
                }
            }
            TeamPrincipal::User { user_id, team_id } => match team_id {
                Some(team_id) => {
                    self.repo
                        .verify_membership(&self.db, *team_id, user_id)
                        .await?;
                    Ok(*team_id)
                }
                None => Ok(self
                    .repo
                    .list_for_user(&self.db, user_id)
                    .await?
                    .into_iter()
                    .next()
                    .ok_or(TeamsError::NotFound)?
                    .id),
            },
        }
    }

    #[must_use = "resolved team id should be used"]
    pub async fn resolve_team_for_owner(
        &self,
        principal: &TeamPrincipal,
        single_user: bool,
    ) -> Result<Uuid, TeamsError> {
        let team_id = self.resolve_team(principal, single_user).await?;
        self.authorize_owner(team_id, principal, single_user)
            .await?;
        Ok(team_id)
    }

    pub async fn ensure_personal_team(
        &self,
        user_id: &str,
        login: &str,
    ) -> Result<Team, TeamsError> {
        let mut tx = self.db.begin().await?;

        self.repo
            .lock_personal_team_creation(&mut *tx, user_id)
            .await?;

        let team = match self.repo.get_personal_team(&mut *tx, user_id).await? {
            Some(team) => team,
            None => {
                self.repo
                    .create_personal_team(&mut *tx, user_id, &format!("{login}'s team"))
                    .await?
            }
        };

        self.repo
            .add_member(&mut *tx, team.id, user_id, "owner")
            .await?;
        tx.commit().await?;

        Ok(team)
    }

    async fn authorize_team_read(
        &self,
        team_id: Uuid,
        principal: &TeamPrincipal,
        single_user: bool,
    ) -> Result<(), TeamsError> {
        match principal {
            TeamPrincipal::Instance { .. } => match single_user {
                true => self.repo.get_by_id(&self.db, team_id).await.map(|_| ()),
                false => Err(TeamsError::AccessDenied),
            },
            TeamPrincipal::User { user_id, .. } => {
                self.repo
                    .verify_membership(&self.db, team_id, user_id)
                    .await
            }
        }
    }

    async fn authorize_owner(
        &self,
        team_id: Uuid,
        principal: &TeamPrincipal,
        single_user: bool,
    ) -> Result<(), TeamsError> {
        match principal {
            TeamPrincipal::Instance { .. } => match single_user {
                true => self.repo.get_by_id(&self.db, team_id).await.map(|_| ()),
                false => Err(TeamsError::AccessDenied),
            },
            TeamPrincipal::User { user_id, .. } => {
                let role = self
                    .repo
                    .get_member_role(&self.db, team_id, user_id)
                    .await?;
                match role.as_str() {
                    "owner" => Ok(()),
                    _ => Err(TeamsError::AccessDenied),
                }
            }
        }
    }
}

fn validate_team_name(name: &str) -> Result<&str, TeamsError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(TeamsError::InvalidOperation(
            "Team name is required".to_owned(),
        ));
    }
    Ok(trimmed)
}

fn default_model_providers(db: PgPool) -> ModelProvidersService {
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
