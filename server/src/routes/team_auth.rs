use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use serde::Deserialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;

#[derive(Clone)]
pub enum TeamPrincipal {
    User {
        user_id: String,
        team_id: Option<Uuid>,
    },
    Instance,
}

#[derive(Deserialize)]
struct Claims {
    sub: String,
    typ: Option<String>,
}

impl FromRequest for TeamPrincipal {
    type Error = Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let token = match req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
        {
            Some(token) => token,
            None => return std::future::ready(Err(AppError::AuthHeaderMissing.into())),
        };

        let state = match req.app_data::<actix_web::web::Data<AppState>>() {
            Some(state) => state,
            None => return std::future::ready(Err(AppError::Internal.into())),
        };

        let decoding_key = jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes());
        let validation = jsonwebtoken::Validation::default();
        let claims = match jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation) {
            Ok(data) => data.claims,
            Err(_) => return std::future::ready(Err(AppError::InvalidToken.into())),
        };

        if claims.sub == "instance" && claims.typ.as_deref().unwrap_or("instance") == "instance" {
            return std::future::ready(Ok(Self::Instance));
        }

        if claims.typ.as_deref() != Some("user") {
            return std::future::ready(Err(AppError::InvalidToken.into()));
        }

        let team_id = req
            .headers()
            .get("X-Team-Id")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| Uuid::parse_str(v).ok());

        std::future::ready(Ok(Self::User {
            user_id: claims.sub,
            team_id,
        }))
    }
}
