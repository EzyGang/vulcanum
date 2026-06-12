use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use serde::Deserialize;
use uuid::Uuid;

use crate::errors::AppError;
use crate::routes::{decode_jwt, parse_team_header};

/// Authenticated principal that may be either a worker or an instance admin.
/// Used for endpoints that both workers (their own jobs) and admins (any job)
/// need to read from.
#[derive(Clone)]
pub enum WorkerOrInstanceAuth {
    Worker {
        worker_id: Uuid,
    },
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

impl FromRequest for WorkerOrInstanceAuth {
    type Error = Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let claims = match decode_jwt::<Claims>(req, AppError::AuthHeaderMissing) {
            Ok(claims) => claims,
            Err(err) => return std::future::ready(Err(err.into())),
        };

        let token_type = claims.typ.as_deref();
        if claims.sub == "instance" && matches!(token_type, None | Some("instance")) {
            return std::future::ready(Ok(Self::Instance));
        }

        if token_type == Some("user") {
            let team_id = match parse_team_header(req) {
                Ok(team_id) => team_id,
                Err(err) => return std::future::ready(Err(err.into())),
            };
            return std::future::ready(Ok(Self::User {
                user_id: claims.sub,
                team_id,
            }));
        }

        match Uuid::parse_str(&claims.sub) {
            Ok(worker_id) if matches!(token_type, None | Some("worker")) => {
                std::future::ready(Ok(Self::Worker { worker_id }))
            }
            _ => std::future::ready(Err(AppError::InvalidToken.into())),
        }
    }
}
