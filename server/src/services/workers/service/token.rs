use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;
use sha2::Digest;
use uuid::Uuid;

use crate::models::workers::model;

#[derive(Serialize)]
struct WorkerClaims {
    sub: String,
    typ: String,
    exp: i64,
}

pub(super) fn generate_random_token() -> String {
    vulcanum_shared::crypto::generate_alphanumeric_string(model::TOKEN_LENGTH)
}

pub(super) fn hash_token(token: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

pub(super) fn build_jwt(
    worker_id: Uuid,
    secret: &str,
) -> Result<(String, DateTime<Utc>), jsonwebtoken::errors::Error> {
    let exp = Utc::now() + Duration::minutes(model::ACCESS_TOKEN_TTL_MINUTES);
    let claims = WorkerClaims {
        sub: worker_id.to_string(),
        typ: "worker".to_owned(),
        exp: exp.timestamp(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok((token, exp))
}
