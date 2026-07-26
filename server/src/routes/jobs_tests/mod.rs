mod authentication;
mod cancellation;
mod events;
mod lifecycle;
mod recent_events;
mod retrieval;

use actix_web::{test, web, App};
use serde_json::json;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::db::dispatcher::DispatchRepository;
use crate::routes;
use crate::test_helpers;

async fn build_state(pool: sqlx::PgPool) -> AppState {
    test_helpers::state::build_state(pool).await
}

fn build_worker_token(worker_id: Uuid) -> String {
    test_helpers::workers::build_worker_token(worker_id)
}

fn build_instance_token() -> String {
    let exp = chrono::Utc::now() + chrono::Duration::minutes(15);
    let claims = serde_json::json!({"sub": "instance", "exp": exp.timestamp()});
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("test-secret".as_bytes()),
    )
    .expect("should build instance token");
    format!("Bearer {token}")
}
