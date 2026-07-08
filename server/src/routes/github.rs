use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::models::auth::model::TeamPrincipal;
use crate::services::github_app::service::GithubInstallState;

pub async fn auth_redirect(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let url = build_install_url_for_principal(&state, auth).await?;

    Ok(HttpResponse::Found()
        .append_header(("Location", url))
        .finish())
}

#[derive(Serialize)]
pub struct AuthUrlResponse {
    pub url: String,
}

pub async fn auth_url(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let url = build_install_url_for_principal(&state, auth).await?;

    Ok(HttpResponse::Ok().json(AuthUrlResponse { url }))
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
    if !matches!(query.setup_action.as_str(), "install" | "update") {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish());
    }

    let install_state = match state
        .github
        .verify_and_consume_state_nonce(&query.state)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, state = %query.state, "github state verification failed");
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/"))
                .finish());
        }
    };

    let install_state = match install_state {
        Some(install_state) => install_state,
        None => {
            tracing::warn!(state = %query.state, "github state nonce not found or expired");
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/"))
                .finish());
        }
    };

    state
        .github
        .create_installation(
            install_state.team_id,
            install_state.user_id.as_deref(),
            query.installation_id,
        )
        .await?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish())
}

pub async fn list_repos(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let repos = state.github.list_repos(team_id).await.map_err(|e| {
        tracing::warn!(error = %e, "list_repos failed");
        AppError::Internal
    })?;

    Ok(HttpResponse::Ok().json(repos))
}

pub async fn get_installation(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let inst = state
        .github
        .get_installation(team_id, state.is_single_user)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "get_installation failed");
            AppError::Internal
        })?;

    Ok(HttpResponse::Ok().json(inst))
}

pub async fn delete_installation(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    state
        .github
        .delete_installation(path.into_inner(), team_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "delete_installation failed");
            AppError::Internal
        })?;

    Ok(HttpResponse::NoContent().finish())
}

async fn build_install_url_for_principal(
    state: &web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<String, AppError> {
    let nonce = Uuid::new_v4().to_string();
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let user_id = match auth {
        TeamPrincipal::User { user_id, .. } => Some(user_id),
        TeamPrincipal::Instance { .. } => None,
    };
    state
        .github
        .save_state_nonce(&nonce, &GithubInstallState { user_id, team_id })
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "failed to save github oauth state");
            AppError::Internal
        })?;

    state.github.install_url(&nonce).await.map_err(|e| {
        tracing::warn!(error = %e, "failed to build install url");
        AppError::Internal
    })
}
