use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::routes::team_auth::TeamPrincipal;
use crate::services::teams::errors::TeamsError;
use crate::services::teams::invite_store::TeamInvitePayload;
use crate::services::teams::model::{
    AcceptTeamInviteResponse, CreateTeamInviteResponse, TeamInvitePreviewResponse,
};
use crate::services::teams::service::TeamsService;

const TEAM_INVITE_TTL_MINUTES: i64 = 30;
const TEAM_INVITE_TOKEN_LENGTH: usize = 64;
const TEAM_INVITE_ROLE: &str = "member";
const GITHUB_PROVIDER: &str = "github";

impl TeamsService {
    pub async fn create_invite_for_principal(
        &self,
        team_id: Uuid,
        principal: &TeamPrincipal,
        single_user: bool,
    ) -> Result<CreateTeamInviteResponse, TeamsError> {
        if single_user {
            return Err(TeamsError::AccessDenied);
        }

        let user_id = match principal {
            TeamPrincipal::User { user_id, .. } => user_id,
            TeamPrincipal::Instance { .. } => return Err(TeamsError::AccessDenied),
        };

        self.authorize_owner(team_id, principal, single_user)
            .await?;
        let token = generate_invite_token();
        let expires_at = Utc::now() + Duration::minutes(TEAM_INVITE_TTL_MINUTES);
        let payload = TeamInvitePayload {
            team_id,
            created_by_user_id: user_id.clone(),
            role: TEAM_INVITE_ROLE.to_owned(),
            expires_at,
        };

        self.invite_store.save(&token, &payload).await?;

        Ok(CreateTeamInviteResponse { token, expires_at })
    }

    pub async fn preview_invite(
        &self,
        token: &str,
        single_user: bool,
    ) -> Result<TeamInvitePreviewResponse, TeamsError> {
        if single_user {
            return Err(TeamsError::InviteInvalid);
        }

        let payload = self
            .invite_store
            .get(token)
            .await?
            .ok_or(TeamsError::InviteInvalid)?;
        self.ensure_invite_team_exists(payload.team_id).await?;

        Ok(TeamInvitePreviewResponse {
            expires_at: payload.expires_at,
        })
    }

    pub async fn accept_invite_for_principal(
        &self,
        token: &str,
        principal: &TeamPrincipal,
        single_user: bool,
    ) -> Result<AcceptTeamInviteResponse, TeamsError> {
        if single_user {
            return Err(TeamsError::InviteInvalid);
        }

        let user_id = match principal {
            TeamPrincipal::User { user_id, .. } => user_id,
            TeamPrincipal::Instance { .. } => return Err(TeamsError::AccessDenied),
        };

        let has_github_identity = self
            .repo
            .user_has_identity(&self.db, user_id, GITHUB_PROVIDER)
            .await?;
        if !has_github_identity {
            return Err(TeamsError::InvalidOperation(
                "GitHub identity is required to accept invites".to_owned(),
            ));
        }

        let payload = self
            .invite_store
            .consume(token)
            .await?
            .ok_or(TeamsError::InviteInvalid)?;
        self.ensure_invite_team_exists(payload.team_id).await?;
        self.repo
            .add_member_preserving_owner(&self.db, payload.team_id, user_id, &payload.role)
            .await?;

        Ok(AcceptTeamInviteResponse {
            team_id: payload.team_id,
        })
    }

    async fn ensure_invite_team_exists(&self, team_id: Uuid) -> Result<(), TeamsError> {
        match self.repo.get_by_id(&self.db, team_id).await {
            Ok(_) => Ok(()),
            Err(TeamsError::NotFound) => Err(TeamsError::InviteInvalid),
            Err(err) => Err(err),
        }
    }
}

fn generate_invite_token() -> String {
    vulcanum_shared::crypto::generate_alphanumeric_string(TEAM_INVITE_TOKEN_LENGTH)
}
