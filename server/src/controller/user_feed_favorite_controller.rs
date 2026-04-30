use crate::api::error::AppError;
use crate::dto::feed_dto::FeedDto;
use crate::entities::{feed, user_feed_favorite};
use crate::AppState;
use axum::extract::{Path, State};
use axum::Json;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use tower_sessions::Session;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/{feed_uuid}",
    tag = "user_feed_favorite",
    responses(
        (status = 200, description = "Feed favorited successfully")
    )
)]
pub async fn favorite_feed(
    State(state): State<AppState>,
    session: Session,
    Path(feed_uuid): Path<Uuid>,
) -> Result<(), AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;

    let favorite = user_feed_favorite::ActiveModel {
        user_uuid: Set(user.id),
        feed_uuid: Set(feed_uuid),
    };

    favorite.insert(&state.db).await?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "/{feed_uuid}",
    tag = "user_feed_favorite",
    responses(
        (status = 200, description = "Feed unfavorited successfully")
    )
)]
pub async fn unfavorite_feed(
    State(state): State<AppState>,
    session: Session,
    Path(feed_uuid): Path<Uuid>,
) -> Result<(), AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;

    user_feed_favorite::Entity::delete_by_id((user.id, feed_uuid))
        .exec(&state.db)
        .await?;

    Ok(())
}

#[utoipa::path(
    get,
    path = "/",
    tag = "user_feed_favorite",
    responses(
        (status = 200, body = Vec<FeedDto>)
    )
)]
pub async fn get_favorite_feeds(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<Vec<FeedDto>>, AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;

    let feeds = feed::Entity::find()
        .inner_join(user_feed_favorite::Entity)
        .filter(user_feed_favorite::Column::UserUuid.eq(user.id))
        .order_by_asc(feed::Column::Name)
        .all(&state.db)
        .await?;

    Ok(Json(feeds.into_iter().map(FeedDto::from).collect()))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(favorite_feed))
        .routes(routes!(unfavorite_feed))
        .routes(routes!(get_favorite_feeds))
}
