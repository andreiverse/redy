use axum::{Json, extract::State};
use sea_orm::EntityTrait;

use crate::{AppState, dto::rss_feed_dto::RssFeedDto, entities::rss_feed};

pub async fn rss_feed_get(State(state): State<AppState>) -> Json<Vec<RssFeedDto>> {
    let feeds = rss_feed::Entity::find().all(&state.db).await.unwrap();

    Json(feeds.into_iter().map(RssFeedDto::from).collect())
}
