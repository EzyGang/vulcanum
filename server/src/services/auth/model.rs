use serde::{Deserialize, Serialize};

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

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
        }
    }
}
