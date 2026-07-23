use uuid::Uuid;

use crate::models::auth::model::TeamPrincipal;
use crate::models::teams::errors::TeamsError;
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
}
