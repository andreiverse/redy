use crate::AppState;
use crate::api::error::AppError;
use crate::dto::worker_dto::{QueueStats, RescheduleRequest, ScheduleResult};
use crate::service::worker_service;
use axum::Json;
use axum::extract::{Path, State};
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
        return Err(AppError::Forbidden(
            "Only admins can view worker stats".to_owned(),
        ));
    }

    let stats = worker_service::get_all_queue_stats(&state.js)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get worker stats: {}", e)))?;

    Ok(Json(stats))
}

#[utoipa::path(
    post,
    path = "/article/{article_uuid}",
    tag = "worker",
    responses(
        (status = 200, body = ScheduleResult)
    )
)]
pub async fn post_article_uuid(
    State(state): State<AppState>,
    Path(uuid): Path<uuid::Uuid>,
    session: Session,
) -> Result<Json<ScheduleResult>, AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    if !user.is_admin {
        return Err(AppError::Forbidden(
            "Only admins can reschedule article ml".to_owned(),
        ));
    }

    let sr: ScheduleResult = worker_service::run_ml_for_uuid(&state.js, uuid)
        .await
        .into();

    Ok(Json(sr))
}

#[utoipa::path(
    post,
    path = "/reschedule",
    tag = "worker",
    request_body = RescheduleRequest,
    responses(
        (status = 200, body = ScheduleResult)
    )
)]
pub async fn post_reschedule(
    State(state): State<AppState>,
    session: Session,
    Json(req): Json<RescheduleRequest>,
) -> Result<Json<ScheduleResult>, AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    if !user.is_admin {
        return Err(AppError::Forbidden(
            "Only admins can reschedule articles".to_owned(),
        ));
    }

    let sr: ScheduleResult = worker_service::reschedule_articles(&state.db, &state.js, req)
        .await
        .into();

    Ok(Json(sr))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_worker_stats))
        .routes(routes!(post_article_uuid))
        .routes(routes!(post_reschedule))
}
