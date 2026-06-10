use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::team_auth::TeamPrincipal;
use crate::services::teams::errors::TeamsError;
use crate::services::teams::model::Team;
use crate::services::teams::repository::TeamsRepository;

#[derive(Clone)]
pub struct TeamsService {
    pub repo: TeamsRepository,
    pub db: PgPool,
}

impl TeamsService {
    pub fn new(repo: TeamsRepository, db: PgPool) -> Self {
        Self { repo, db }
    }

    pub async fn list_for_user(&self, user_id: &str) -> Result<Vec<Team>, TeamsError> {
        self.repo.list_for_user(&self.db, user_id).await
    }

    pub async fn resolve_team(
        &self,
        principal: &TeamPrincipal,
        single_user: bool,
    ) -> Result<Uuid, TeamsError> {
        match principal {
            TeamPrincipal::Instance => {
                if !single_user {
                    return Err(TeamsError::AccessDenied);
                }
                Ok(self.repo.get_default_team(&self.db).await?.id)
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
        let teams = self.repo.list_for_user(&self.db, user_id).await?;
        match teams
            .into_iter()
            .find(|team| team.personal_user_id.as_deref() == Some(user_id))
        {
            Some(team) => Ok(team),
            None => {
                let team = self
                    .repo
                    .create_personal_team(&self.db, user_id, &format!("{login}'s team"))
                    .await?;
                self.repo
                    .add_member(&self.db, team.id, user_id, "owner")
                    .await?;
                Ok(team)
            }
        }
    }
}
