use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::errors::AppError;

#[derive(Deserialize)]
struct InstanceClaims {
    sub: String,
}

pub struct InstanceAuth;

impl FromRequest for InstanceAuth {
    type Error = Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));

        let token = match header {
            Some(t) => t,
            None => {
                return std::future::ready(Err(AppError::AuthHeaderMissing.into()));
            }
        };

        let state = match req.app_data::<actix_web::web::Data<AppState>>() {
            Some(s) => s,
            None => return std::future::ready(Err(AppError::Internal.into())),
        };

        let decoding_key = DecodingKey::from_secret(state.jwt_secret.as_bytes());
        let validation = Validation::default();

        match decode::<InstanceClaims>(token, &decoding_key, &validation) {
            Ok(data) if data.claims.sub == "instance" => std::future::ready(Ok(InstanceAuth)),
            _ => std::future::ready(Err(AppError::InvalidToken.into())),
        }
    }
}
