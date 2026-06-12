use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;

use crate::services::auth::errors::AuthError;
use crate::services::auth::service::AuthService;

const INSTANCE_TOKEN_TTL_HOURS: i64 = 24;

#[derive(Serialize)]
struct InstanceClaims {
    sub: String,
    typ: String,
    exp: usize,
    iat: usize,
}

impl AuthService {
    pub fn instance_login(&self, password: &str) -> Result<String, AuthError> {
        if !self.is_single_user {
            return Err(AuthError::InstanceLoginDisabled);
        }

        if password != self.instance_password {
            return Err(AuthError::InvalidPassword);
        }

        let now = Utc::now();
        let claims = InstanceClaims {
            sub: "instance".to_owned(),
            typ: "instance".to_owned(),
            iat: now.timestamp() as usize,
            exp: (now + Duration::hours(INSTANCE_TOKEN_TTL_HOURS)).timestamp() as usize,
        };

        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_bytes());
        encode(&Header::default(), &claims, &encoding_key).map_err(|_| AuthError::InvalidToken)
    }
}
