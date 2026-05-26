use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub created_at: String,
    pub last_login_at: Option<String>,
}
