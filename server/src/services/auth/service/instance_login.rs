use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;
use sha2::Sha256;
use uuid::Uuid;
use vulcanum_shared::api_types::AuthTokenResponse;

use crate::models::auth::errors::AuthError;
use crate::services::auth::service::refresh::{generate_random_token, hash_token};
use crate::services::auth::service::AuthService;

const INSTANCE_ACCESS_TOKEN_TTL_MINUTES: i64 = 15;
const INSTANCE_REFRESH_TOKEN_TTL_DAYS: i64 = 1;
const INSTANCE_REFRESH_TOKEN_LENGTH: usize = 64;
pub(super) const INSTANCE_REFRESH_TOKEN_PREFIX: &str = "irt1_";
const PASSWORD_FINGERPRINT_DOMAIN: &[u8] = b"vulcanum:instance-refresh-password:v1\0";

#[derive(Serialize)]
struct InstanceClaims {
    sub: String,
    typ: String,
    iat: usize,
    exp: usize,
    jti: String,
}

impl AuthService {
    pub async fn instance_login(&self, password: &str) -> Result<AuthTokenResponse, AuthError> {
        if !self.is_single_user {
            return Err(AuthError::InstanceLoginDisabled);
        }

        if !constant_time_eq(password.as_bytes(), self.instance_password.as_bytes()) {
            return Err(AuthError::InvalidPassword);
        }

        self.issue_instance_token_pair().await
    }

    pub async fn refresh_instance_token(
        &self,
        refresh_token: &str,
    ) -> Result<AuthTokenResponse, AuthError> {
        if !self.is_single_user || !refresh_token.starts_with(INSTANCE_REFRESH_TOKEN_PREFIX) {
            return Err(AuthError::InvalidRefreshToken);
        }
        let password_fingerprint = self.instance_password_fingerprint()?;
        self.repo
            .revoke_mismatched_instance_refresh_tokens(&self.db, &password_fingerprint)
            .await?;

        let access_token = self.build_instance_jwt()?;
        let new_refresh_token = create_instance_refresh_token();
        let new_refresh_token_hash = hash_token(&new_refresh_token);
        let refresh_expires_at = Utc::now() + Duration::days(INSTANCE_REFRESH_TOKEN_TTL_DAYS);

        self.repo
            .rotate_instance_refresh_token(
                &self.db,
                &hash_token(refresh_token),
                &new_refresh_token_hash,
                &password_fingerprint,
                refresh_expires_at,
            )
            .await?;

        Ok(AuthTokenResponse {
            access_token,
            refresh_token: new_refresh_token,
            refresh_expires_at,
        })
    }

    pub async fn revoke_instance_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<(), AuthError> {
        self.repo
            .revoke_instance_refresh_token(&self.db, &hash_token(refresh_token))
            .await
    }

    async fn issue_instance_token_pair(&self) -> Result<AuthTokenResponse, AuthError> {
        let access_token = self.build_instance_jwt()?;
        let refresh_token = create_instance_refresh_token();
        let refresh_expires_at = Utc::now() + Duration::days(INSTANCE_REFRESH_TOKEN_TTL_DAYS);

        self.repo
            .create_instance_refresh_token(
                &self.db,
                &hash_token(&refresh_token),
                &self.instance_password_fingerprint()?,
                refresh_expires_at,
            )
            .await?;

        Ok(AuthTokenResponse {
            access_token,
            refresh_token,
            refresh_expires_at,
        })
    }

    fn build_instance_jwt(&self) -> Result<String, AuthError> {
        let now = Utc::now();
        let claims = InstanceClaims {
            sub: "instance".to_owned(),
            typ: "instance".to_owned(),
            iat: now.timestamp() as usize,
            exp: (now + Duration::minutes(INSTANCE_ACCESS_TOKEN_TTL_MINUTES)).timestamp() as usize,
            jti: Uuid::new_v4().to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AuthError::InvalidToken)
    }

    fn instance_password_fingerprint(&self) -> Result<String, AuthError> {
        let mut mac = Hmac::<Sha256>::new_from_slice(self.jwt_secret.as_bytes())
            .map_err(|_| AuthError::InvalidToken)?;
        mac.update(PASSWORD_FINGERPRINT_DOMAIN);
        mac.update(self.instance_password.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }
}

fn create_instance_refresh_token() -> String {
    format!(
        "{INSTANCE_REFRESH_TOKEN_PREFIX}{}",
        generate_random_token(INSTANCE_REFRESH_TOKEN_LENGTH)
    )
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let max_len = left.len().max(right.len());
    let mut diff = left.len() ^ right.len();
    for index in 0..max_len {
        let left_byte = left.get(index).copied().unwrap_or(0);
        let right_byte = right.get(index).copied().unwrap_or(0);
        diff |= usize::from(left_byte ^ right_byte);
    }
    diff == 0
}
