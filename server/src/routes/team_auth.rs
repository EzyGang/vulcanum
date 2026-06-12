use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use serde::Deserialize;
use uuid::Uuid;

use crate::errors::AppError;
use crate::routes::{decode_jwt, parse_team_header};

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
        let claims = match decode_jwt::<Claims>(req, AppError::AuthHeaderMissing) {
            Ok(claims) => claims,
            Err(err) => return std::future::ready(Err(err.into())),
        };

        if claims.sub == "instance" && claims.typ.as_deref().unwrap_or("instance") == "instance" {
            return std::future::ready(Ok(Self::Instance));
        }

        if claims.typ.as_deref() != Some("user") {
            return std::future::ready(Err(AppError::InvalidToken.into()));
        }

        let team_id = match parse_team_header(req) {
            Ok(team_id) => team_id,
            Err(err) => return std::future::ready(Err(err.into())),
        };

        std::future::ready(Ok(Self::User {
            user_id: claims.sub,
            team_id,
        }))
    }
}
