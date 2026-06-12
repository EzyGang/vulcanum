pub mod invites;

use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::team_auth::TeamPrincipal;
use crate::services::teams::errors::TeamsError;
use crate::services::teams::invite_store::{InMemoryTeamInviteStore, TeamInviteStore};
use crate::services::teams::model::{ProviderIdentity, Team, TeamMemberInfo};
use crate::services::teams::repository::TeamsRepository;

#[derive(Clone)]
pub struct TeamsService {
    pub repo: TeamsRepository,
    pub db: PgPool,
    pub invite_store: Arc<dyn TeamInviteStore>,
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
        Self {
            repo,
            db,
            invite_store,
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
        name: &str,
        principal: &TeamPrincipal,
        single_user: bool,
    ) -> Result<Team, TeamsError> {
        let name = validate_team_name(name)?;
        self.authorize_owner(team_id, principal, single_user)
            .await?;
        self.repo.update_name(&self.db, team_id, name).await
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
