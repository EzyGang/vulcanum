mod auth;
pub mod invites;
mod model_settings;

use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::team_auth::TeamPrincipal;
use crate::services::model_providers::service::ModelProvidersService;
use crate::services::teams::errors::TeamsError;
use crate::services::teams::invite_store::{InMemoryTeamInviteStore, TeamInviteStore};
use crate::services::teams::model::{ProviderIdentity, Team, TeamMemberInfo};
use crate::services::teams::repository::TeamsRepository;

use self::model_settings::default_model_providers;

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
