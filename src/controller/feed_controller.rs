use crate::dto::article_dto::ArticleDto;
use crate::entities::feed;
use crate::service::rss_fetcher_service::{rss_fetch};
use crate::{AppState, dto::feed_dto::FeedDto, entities::rss_feed};
use axum::extract::Path;
use axum::{Json, extract::State};
use sea_orm::{EntityTrait, TryIntoModel};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/",
    tag = "feed",
    responses(
        (status=200, body=Vec<FeedDto>)
    )
)]
pub async fn feed_get(State(state): State<AppState>) -> Json<Vec<FeedDto>> {
    let feeds = feed::Entity::find().all(&state.db).await.unwrap();

    Json(feeds.into_iter().map(FeedDto::from).collect())
}

#[utoipa::path(
    get,
    path = "/{feed_uuid}",
    tag = "feed",
    responses(
        (status=200, body=FeedDto)
    )
)]
pub async fn feed_get_by_uuid(
    State(state): State<AppState>,
    Path(feed_uuid): Path<Uuid>,
) -> Json<FeedDto> {
    let feed = feed::Entity::find_by_id(feed_uuid)
        .one(&state.db)
        .await
        .unwrap()
        .unwrap();

    Json(FeedDto::from(feed))
}

#[utoipa::path(
    get,
    path = "/{feed_uuid}/fetch",
    tag = "feed",
    responses(
        (status=200, body=Vec<ArticleDto>)
    )
)]
pub async fn feed_fetch_by_uuid(
    State(state): State<AppState>,
    Path(feed_uuid): Path<Uuid>,
) -> Json<Vec<ArticleDto>> {
    let feed = feed::Entity::find_by_id(feed_uuid)
        .one(&state.db)
        .await
        .unwrap()
        .unwrap();

    match feed.feed_type {
        crate::entities::sea_orm_active_enums::FeedType::Rss => {
            return Json(
                rss_fetch(feed)
                    .await
                    .unwrap()
                    .iter()
                    // todo: check if clone is avoidable
                    .map(|f| f.clone().try_into_model().unwrap())
                    .map(ArticleDto::from)
                    .collect(),
            );
        }
    }
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(feed_get))
        .routes(routes!(feed_get_by_uuid))
        .routes(routes!(feed_fetch_by_uuid))
}
