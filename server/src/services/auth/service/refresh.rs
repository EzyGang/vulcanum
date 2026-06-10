use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;
use sha2::Digest;

use crate::services::auth::errors::AuthError;
use crate::services::auth::model::AuthTokenResponse;
use crate::services::auth::service::AuthService;

const USER_ACCESS_TOKEN_TTL_MINUTES: i64 = 15;
const USER_REFRESH_TOKEN_TTL_DAYS: i64 = 30;
const USER_REFRESH_TOKEN_LENGTH: usize = 64;

#[derive(Serialize)]
struct UserClaims {
    sub: String,
    typ: String,
    exp: usize,
    iat: usize,
}

impl AuthService {
    pub async fn issue_user_token_pair(
        &self,
        user_id: &str,
    ) -> Result<AuthTokenResponse, AuthError> {
        let access_token = self.build_user_jwt(user_id)?;
        let refresh_token = generate_random_token(USER_REFRESH_TOKEN_LENGTH);
        let refresh_token_hash = hash_token(&refresh_token);
        let refresh_expires_at = Utc::now() + Duration::days(USER_REFRESH_TOKEN_TTL_DAYS);

        self.repo
            .create_refresh_token(&self.db, user_id, &refresh_token_hash, refresh_expires_at)
            .await?;

        Ok(AuthTokenResponse {
            access_token,
            refresh_token,
            refresh_expires_at,
        })
    }

    pub async fn refresh_user_token(
        &self,
        refresh_token: &str,
    ) -> Result<AuthTokenResponse, AuthError> {
        let refresh_token_hash = hash_token(refresh_token);
        let new_refresh_token = generate_random_token(USER_REFRESH_TOKEN_LENGTH);
        let new_refresh_token_hash = hash_token(&new_refresh_token);
        let refresh_expires_at = Utc::now() + Duration::days(USER_REFRESH_TOKEN_TTL_DAYS);
        let user_id = self
            .repo
            .rotate_refresh_token(
                &self.db,
                &refresh_token_hash,
                &new_refresh_token_hash,
                refresh_expires_at,
            )
            .await?;
        let access_token = self.build_user_jwt(&user_id)?;

        Ok(AuthTokenResponse {
            access_token,
            refresh_token: new_refresh_token,
            refresh_expires_at,
        })
    }

    pub async fn revoke_user_refresh_token(&self, refresh_token: &str) -> Result<(), AuthError> {
        let refresh_token_hash = hash_token(refresh_token);
        self.repo
            .revoke_refresh_token(&self.db, &refresh_token_hash)
            .await
    }

    pub fn build_user_jwt(&self, user_id: &str) -> Result<String, AuthError> {
        let now = Utc::now();
        let claims = UserClaims {
            sub: user_id.to_owned(),
            typ: "user".to_owned(),
            iat: now.timestamp() as usize,
            exp: (now + Duration::minutes(USER_ACCESS_TOKEN_TTL_MINUTES)).timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AuthError::InvalidToken)
    }
}

fn generate_random_token(length: usize) -> String {
    vulcanum_shared::crypto::generate_alphanumeric_string(length)
}

fn hash_token(token: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
