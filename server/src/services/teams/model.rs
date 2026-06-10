use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub personal_user_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TeamMember {
    pub team_id: Uuid,
    pub user_id: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserIdentity {
    pub user_id: String,
    pub provider_user_id: String,
    pub provider_login: String,
}
