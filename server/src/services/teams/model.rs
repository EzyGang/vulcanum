use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::util::serde::deserialize_nullable_string;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub personal_user_id: Option<String>,
    pub prompt_template: String,
    pub agents_md: String,
    pub primary_model_provider_key: Option<String>,
    pub primary_model_id: Option<String>,
    pub small_model_provider_key: Option<String>,
    pub small_model_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TeamMember {
    pub team_id: Uuid,
    pub user_id: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TeamMemberInfo {
    pub team_id: Uuid,
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTeamRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub prompt_template: Option<String>,
    #[serde(default)]
    pub agents_md: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub primary_model_provider_key: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub primary_model_id: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub small_model_provider_key: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub small_model_id: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateTeamInviteResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamInvitePreviewResponse {
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AcceptTeamInviteResponse {
    pub team_id: Uuid,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserIdentity {
    pub user_id: String,
    pub provider_user_id: String,
    pub provider_login: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ProviderIdentity {
    pub provider: String,
    pub provider_user_id: String,
    pub provider_login: String,
    pub provider_verified_at: Option<DateTime<Utc>>,
}
