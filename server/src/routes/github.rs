use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::routes::instance_auth::InstanceAuth;

pub async fn auth_redirect(
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let nonce = Uuid::new_v4().to_string();
    state.github.save_state_nonce(&nonce).await.map_err(|e| {
        tracing::warn!(error = %e, "failed to save github oauth state");
        AppError::Internal
    })?;

    let url = state.github.install_url(&nonce).await.map_err(|e| {
        tracing::warn!(error = %e, "failed to build install url");
        AppError::Internal
    })?;

    Ok(HttpResponse::Found()
        .append_header(("Location", url))
        .finish())
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub installation_id: i64,
    pub setup_action: String,
    pub state: String,
}

pub async fn callback(
    state: web::Data<AppState>,
    query: web::Query<CallbackQuery>,
) -> Result<HttpResponse, AppError> {
    let valid = state
        .github
        .verify_and_consume_state_nonce(&query.state)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "state nonce verification failed");
            AppError::Internal
        })?;

    if !valid {
        return Err(AppError::BadRequest("invalid or expired state".to_string()));
    }

    if query.setup_action != "install" {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/projects"))
            .finish());
    }

    state
        .github
        .create_installation(query.installation_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, installation_id = query.installation_id, "create_installation failed");
            AppError::Internal
        })?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/projects"))
        .finish())
}

pub async fn list_repos(
    state: web::Data<AppState>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let repos = state.github.list_repos().await.map_err(|e| {
        tracing::warn!(error = %e, "list_repos failed");
        AppError::Internal
    })?;

    Ok(HttpResponse::Ok().json(repos))
}

pub async fn get_installation(
    state: web::Data<AppState>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    let inst = state.github.get_installation().await.map_err(|e| {
        tracing::warn!(error = %e, "get_installation failed");
        AppError::Internal
    })?;

    Ok(HttpResponse::Ok().json(inst))
}

pub async fn delete_installation(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    _auth: InstanceAuth,
) -> Result<HttpResponse, AppError> {
    state
        .github
        .delete_installation(path.into_inner())
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "delete_installation failed");
            AppError::Internal
        })?;

    Ok(HttpResponse::NoContent().finish())
}
