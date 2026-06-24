use uuid::Uuid;

use crate::routes::team_auth::TeamPrincipal;
use crate::services::teams::errors::TeamsError;
use crate::services::teams::service::TeamsService;

impl TeamsService {
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

    pub(super) async fn authorize_team_read(
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

    pub(super) async fn authorize_owner(
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
