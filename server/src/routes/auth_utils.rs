use actix_web::{web, HttpRequest};
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;

pub(crate) fn parse_team_header(req: &HttpRequest) -> Result<Option<Uuid>, AppError> {
    let header = match req.headers().get("X-Team-Id") {
        Some(header) => header,
        None => return Ok(None),
    };

    let value = header
        .to_str()
        .map_err(|_| AppError::BadRequest("Invalid X-Team-Id header".to_owned()))?;
    Uuid::parse_str(value)
        .map(Some)
        .map_err(|_| AppError::BadRequest("Invalid X-Team-Id header".to_owned()))
}

pub(crate) fn decode_jwt<T>(
    req: &HttpRequest,
    missing_header_error: AppError,
) -> Result<T, AppError>
where
    T: DeserializeOwned,
{
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(missing_header_error)?;

    let state = req
        .app_data::<web::Data<AppState>>()
        .ok_or(AppError::Internal)?;
    let decoding_key = jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes());
    let validation = jsonwebtoken::Validation::default();

    jsonwebtoken::decode::<T>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|_| AppError::InvalidToken)
}
