use crate::service::rss_fetcher_service::{RssNews, rss_fetch};
use crate::{AppState, dto::rss_feed_dto::RssFeedDto, entities::rss_feed};
use axum::extract::Path;
use axum::{Json, extract::State};
use sea_orm::{EntityTrait};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/",
    tag = "rss_feed",
    responses(
        (status=200, body=Vec<RssFeedDto>)
    )
)]
pub async fn rss_feed_get(State(state): State<AppState>) -> Json<Vec<RssFeedDto>> {
    let feeds = rss_feed::Entity::find().all(&state.db).await.unwrap();

    Json(feeds.into_iter().map(RssFeedDto::from).collect())
}


#[utoipa::path(
    get,
    path = "/{rss_feed_uuid}",
    tag = "rss_feed",
    responses(
        (status=200, body=RssFeedDto)
    )
)]
pub async fn rss_feed_get_by_uuid(
    State(state): State<AppState>,
    Path(rss_feed_uuid): Path<Uuid>,
) -> Json<RssFeedDto> {
    let feed = rss_feed::Entity::find_by_id(rss_feed_uuid)
        .one(&state.db)
        .await
        .unwrap()
        .unwrap();

    Json(RssFeedDto::from(feed))
}

#[utoipa::path(
    get,
    path = "/{rss_feed_uuid}/fetch",
    tag = "rss_feed",
    responses(
        (status=200, body=Vec<RssNews>)
    )
)]
pub async fn rss_feed_fetch_by_uuid(
    State(state): State<AppState>,
    Path(rss_feed_uuid): Path<Uuid>,
) -> Json<Vec<RssNews>> {
    let feed = rss_feed::Entity::find_by_id(rss_feed_uuid)
        .one(&state.db)
        .await
        .unwrap()
        .unwrap();

    let rss_feed = rss_fetch(feed.url).await.unwrap();

    Json(rss_feed)
}


pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(rss_feed_get))
        .routes(routes!(rss_feed_get_by_uuid))
        .routes(routes!(rss_feed_fetch_by_uuid))
}