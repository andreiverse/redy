use crate::api::error::AppError;
use crate::dto::worker_dto::QueueStats;
use crate::service::worker_service;
use crate::AppState;
use axum::extract::State;
use axum::Json;
use tower_sessions::Session;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/stats",
    tag = "worker",
    responses(
        (status = 200, body = Vec<QueueStats>)
    )
)]
pub async fn get_worker_stats(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<Vec<QueueStats>>, AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    if !user.is_admin {
        return Err(AppError::Forbidden("Only admins can view worker stats".to_owned()));
    }

    let stats = worker_service::get_all_queue_stats(&state.js).await
        .map_err(|e| AppError::Internal(format!("Failed to get worker stats: {}", e)))?;

    Ok(Json(stats))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_worker_stats))
}
