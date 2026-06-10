use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::teams::model::Team;
use crate::services::users::model::User;

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
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct InstanceLoginRequest {
    pub password: String,
}

#[derive(Serialize)]
pub struct InstanceLoginResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct GithubCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Serialize)]
pub struct AuthModeResponse {
    pub is_single_user: bool,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub user: UserInfo,
    pub teams: Vec<TeamInfo>,
}

#[derive(Serialize)]
pub struct TeamInfo {
    pub id: Uuid,
    pub name: String,
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
