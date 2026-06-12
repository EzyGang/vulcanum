use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
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

impl FromRequest for WorkerOrInstanceAuth {
    type Error = Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let claims = match decode_jwt::<serde_json::Value>(req, AppError::AuthHeaderMissing) {
            Ok(claims) => claims,
            Err(err) => return std::future::ready(Err(err.into())),
        };

        match claims.get("sub").and_then(|s| s.as_str()) {
            Some(sub) => {
                let token_type = claims.get("typ").and_then(|s| s.as_str());
                if sub == "instance" && matches!(token_type, None | Some("instance")) {
                    return std::future::ready(Ok(Self::Instance));
                }
                if token_type == Some("user") {
                    let team_id = match parse_team_header(req) {
                        Ok(team_id) => team_id,
                        Err(err) => return std::future::ready(Err(err.into())),
                    };
                    return std::future::ready(Ok(Self::User {
                        user_id: sub.to_owned(),
                        team_id,
                    }));
                }
                match Uuid::parse_str(sub) {
                    Ok(worker_id) if matches!(token_type, None | Some("worker")) => {
                        std::future::ready(Ok(Self::Worker { worker_id }))
                    }
                    _ => std::future::ready(Err(AppError::InvalidToken.into())),
                }
            }
            None => std::future::ready(Err(AppError::InvalidToken.into())),
        }
    }
}
