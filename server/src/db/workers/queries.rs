mod capacity;
mod registration;
mod status;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::db::workers::{map_sqlx_error, WorkersRepository};
use crate::models::workers::errors::WorkersError;
use crate::models::workers::model::{Worker, WorkerStatus};

pub struct CreateWorkerParams<'a> {
    pub team_id: Uuid,
    pub name: &'a str,
    pub refresh_token_hash: &'a str,
    pub refresh_expires_at: DateTime<Utc>,
    pub capabilities: &'a serde_json::Value,
    pub max_concurrent_jobs: i32,
}
