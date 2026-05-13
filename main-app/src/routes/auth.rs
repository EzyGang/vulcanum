use actix_web::{web, HttpResponse};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::db;
use crate::errors::AppError;

const TOKEN_TTL_MINUTES: i64 = 15;

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    token: String,
}

#[derive(Serialize)]
struct LoginResponse {
    message: String,
}

#[derive(Serialize)]
struct VerifyResponse {
    message: String,
    user: UserInfo,
}

#[derive(Serialize)]
struct UserInfo {
    id: String,
    email: String,
}

pub async fn login(body: web::Json<LoginRequest>) -> Result<HttpResponse, AppError> {
    let pool = db::db_config::pool();
    let user = db::users::find_or_create_user(pool, &body.email).await?;

    let token = generate_token();
    db::token_store::insert_token(&token, &user.id, TOKEN_TTL_MINUTES);

    log::info!(
        "Magic link for {}: http://localhost:8080/api/v1/auth/verify?token={}",
        body.email,
        token
    );

    Ok(HttpResponse::Ok().json(LoginResponse {
        message: "If the email is registered, a magic link has been logged".to_owned(),
    }))
}

pub async fn verify(query: web::Query<VerifyQuery>) -> Result<HttpResponse, AppError> {
    let user_id = db::token_store::consume_token(&query.token).ok_or(AppError::InvalidToken)?;

    let pool = db::db_config::pool();
    let user = db::users::find_user_by_id(pool, &user_id).await?;
    db::users::update_last_login(pool, &user.id).await?;

    Ok(HttpResponse::Ok().json(VerifyResponse {
        message: "Logged in successfully".to_owned(),
        user: UserInfo {
            id: user.id,
            email: user.email,
        },
    }))
}

fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}
