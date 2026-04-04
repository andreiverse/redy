use axum::response::{IntoResponse, Response};
use tower_sessions::Session;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

#[utoipa::path(
    get,
    path="/",
    tag="sessions",
    responses(
        (status=200)
    )
)]
pub async fn sessions_get(session: Session) -> Response {
    session.insert("yes", "no").await.unwrap();
    session.save().await.unwrap();

    let id = session
        .id()
        .map(|id| id.0.to_string())
        .unwrap_or_else(|| "session pending".to_string());

    id.into_response()
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(sessions_get))
}