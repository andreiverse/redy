use axum::Json;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use openidconnect::{Nonce, PkceCodeVerifier};
use sea_orm::EntityTrait;
use serde::Deserialize;
use tower_sessions::Session;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;
use crate::api::error::AppError;
use crate::dto::user_dto::UserDto;
use crate::entities::user;

#[derive(Deserialize)]
pub struct LoginQuery {
    redirect_to_frontend: Option<bool>,
}

#[derive(Deserialize)]
pub struct AuthCallbackQuery {
    code: String,
    state: String,
}

#[utoipa::path(
    get,
    path = "/login",
    tag = "auth",
    params(
        ("redirect_to_frontend" = Option<bool>, Query, description = "Redirect to frontend after successful login")
    ),
    responses(
        (status = 302, description = "Redirect to OIDC provider")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<LoginQuery>,
) -> Result<Redirect, AppError> {
    let (auth_url, csrf_token, nonce, pkce_verifier) = state.auth_service.get_auth_url();

    session.insert("csrf_token", csrf_token).await.unwrap();
    session.insert("nonce", nonce).await.unwrap();
    session
        .insert("pkce_verifier", pkce_verifier)
        .await
        .unwrap();

    if let Some(true) = query.redirect_to_frontend {
        session.insert("redirect_to_frontend", true).await.unwrap();
    }

    Ok(Redirect::to(&auth_url))
}

#[utoipa::path(
    get,
    path = "/callback",
    tag = "auth",
    responses(
        (status = 200, body = UserDto),
        (status = 302, description = "Redirect to frontend if redirect_to_frontend was true"),
        (status = 400, description = "Invalid state or code")
    )
)]
pub async fn callback(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<AuthCallbackQuery>,
) -> Result<Response, AppError> {
    let csrf_token: String = session
        .get("csrf_token")
        .await
        .unwrap()
        .ok_or_else(|| AppError::BadRequest("Missing CSRF token in session".to_string()))?;

    if query.state != csrf_token {
        return Err(AppError::BadRequest("Invalid state".to_string()));
    }

    let nonce: Nonce = session
        .get("nonce")
        .await
        .unwrap()
        .ok_or_else(|| AppError::Internal("Missing nonce in session".to_string()))?;

    let pkce_verifier: PkceCodeVerifier = session
        .get("pkce_verifier")
        .await
        .unwrap()
        .ok_or_else(|| AppError::Internal("Missing pkce_verifier in session".to_string()))?;

    let user = state
        .auth_service
        .authenticate(query.code, nonce, pkce_verifier)
        .await?;

    session.insert("user_id", user.id).await.unwrap();

    let redirect_to_frontend: Option<bool> = session.get("redirect_to_frontend").await.unwrap();

    session.remove::<String>("csrf_token").await.unwrap();
    session.remove::<Nonce>("nonce").await.unwrap();
    session
        .remove::<PkceCodeVerifier>("pkce_verifier")
        .await
        .unwrap();
    session
        .remove::<bool>("redirect_to_frontend")
        .await
        .unwrap();

    if let Some(true) = redirect_to_frontend {
        Ok(Redirect::to(&state.frontend_url).into_response())
    } else {
        Ok(Json(UserDto::from(user)).into_response())
    }
}

#[utoipa::path(
    get,
    path = "/me",
    tag = "auth",
    responses(
        (status = 200, body = UserDto),
        (status = 401, description = "Not logged in")
    )
)]
pub async fn me(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<UserDto>, AppError> {
    let user_id: uuid::Uuid = session
        .get("user_id")
        .await
        .unwrap()
        .ok_or_else(|| AppError::Auth("Not logged in".to_string()))?;

    let user = user::Entity::find_by_id(user_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::Auth("User not found".to_string()))?;

    Ok(Json(UserDto::from(user)))
}

#[utoipa::path(
    post,
    path = "/logout",
    tag = "auth",
    responses(
        (status = 200, description = "Successfully logged out")
    )
)]
pub async fn logout(session: Session) -> Result<(), AppError> {
    session.clear().await;
    Ok(())
}

#[utoipa::path(
    delete,
    path = "/me",
    tag = "auth",
    responses(
        (status = 200, description = "Successfully deleted user"),
        (status = 401, description = "Not logged in")
    )
)]
pub async fn delete_me(State(state): State<AppState>, session: Session) -> Result<(), AppError> {
    let user_id: uuid::Uuid = session
        .get("user_id")
        .await
        .unwrap()
        .ok_or_else(|| AppError::Auth("Not logged in".to_string()))?;

    user::Entity::delete_by_id(user_id).exec(&state.db).await?;

    session.clear().await;
    Ok(())
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(login))
        .routes(routes!(callback))
        .routes(routes!(me))
        .routes(routes!(logout))
        .routes(routes!(delete_me))
}
