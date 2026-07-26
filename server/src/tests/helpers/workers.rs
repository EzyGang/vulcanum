use super::{teams, Uuid, DEFAULT_TEAM_ID};

pub async fn insert_worker(pool: &sqlx::PgPool, name: &str) -> Uuid {
    teams::ensure_default_team(pool).await;
    insert_worker_for_team(pool, DEFAULT_TEAM_ID, name).await
}

pub async fn insert_worker_for_team(pool: &sqlx::PgPool, team_id: Uuid, name: &str) -> Uuid {
    if team_id == DEFAULT_TEAM_ID {
        teams::ensure_default_team(pool).await;
    }

    let id = Uuid::new_v4();
    let hash = hex::encode([0u8; 32]);

    sqlx::query!(
        "INSERT INTO workers (id, team_id, name, refresh_token_hash, refresh_expires_at, status) VALUES ($1, $2, $3, $4, NOW() + INTERVAL '30 days', 'idle'::worker_status)",
        id,
        team_id,
        name,
        hash,
    )
    .execute(pool)
    .await
    .expect("Should insert worker");

    id
}
pub fn build_worker_token(worker_id: Uuid) -> String {
    let exp = chrono::Utc::now() + chrono::Duration::minutes(15);
    let claims =
        serde_json::json!({"sub": worker_id.to_string(), "typ": "worker", "exp": exp.timestamp()});
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("test-secret".as_bytes()),
    )
    .expect("should build token");
    format!("Bearer {token}")
}
