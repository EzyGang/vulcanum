use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use serde::Deserialize;

use crate::errors::AppError;
use crate::routes::auth_utils::decode_jwt;

#[derive(Deserialize)]
struct InstanceClaims {
    sub: String,
    typ: Option<String>,
}

pub struct InstanceAuth;

impl FromRequest for InstanceAuth {
    type Error = Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        match decode_jwt::<InstanceClaims>(req, AppError::AuthHeaderMissing) {
            Ok(claims)
                if claims.sub == "instance"
                    && matches!(claims.typ.as_deref(), None | Some("instance")) =>
            {
                std::future::ready(Ok(InstanceAuth))
            }
            _ => std::future::ready(Err(AppError::InvalidToken.into())),
        }
    }
}
