use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vulcanum_shared::api::wire::AuthTokenResponse;

use crate::models::teams::model::{ProviderIdentity, Team};
use crate::models::users::model::User;

#[derive(Clone)]
pub enum TeamPrincipal {
    User {
        user_id: String,
        team_id: Option<Uuid>,
    },
    Instance {
        team_id: Option<Uuid>,
    },
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    pub token: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub message: String,
    pub user: UserInfo,
    #[serde(flatten)]
    pub token_pair: AuthTokenResponse,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
}

#[derive(Deserialize)]
pub struct GithubStartQuery {
    pub return_to: Option<String>,
}

#[derive(Deserialize)]
pub struct GithubCallbackQuery {
    pub code: String,
    pub state: String,
}

pub enum GithubCallbackResult {
    Login {
        token_pair: AuthTokenResponse,
        return_to: String,
    },
    IdentityLinked {
        return_to: String,
    },
}

#[derive(Serialize)]
pub struct MeResponse {
    pub user: UserInfo,
    pub teams: Vec<TeamInfo>,
    pub identities: Vec<IdentityInfo>,
}

#[derive(Serialize)]
pub struct TeamInfo {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize)]
pub struct IdentityInfo {
    pub provider: String,
    pub provider_user_id: String,
    pub login: String,
    pub verified_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
        }
    }
}

impl From<Team> for TeamInfo {
    fn from(team: Team) -> Self {
        Self {
            id: team.id,
            name: team.name,
        }
    }
}

impl From<ProviderIdentity> for IdentityInfo {
    fn from(identity: ProviderIdentity) -> Self {
        Self {
            provider: identity.provider,
            provider_user_id: identity.provider_user_id,
            login: identity.provider_login,
            verified_at: identity.provider_verified_at,
        }
    }
}
