use crate::dto::article_dto::ArticleDto;
use crate::entities::article;
use crate::AppState;
use axum::extract::{Path, Query};
use axum::{Json, extract::State};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[derive(IntoParams, Deserialize)]
pub struct ArticleGetParams {
    pub feed_uuid: Option<Uuid>,
}

#[utoipa::path(
    get,
    path = "/{article_uuid}",
    tag = "article",
    responses(
        (status=200, body=ArticleDto)
    )
)]
pub async fn article_get_by_uuid(
    State(state): State<AppState>,
    Path(article_uuid): Path<Uuid>,
) -> Json<ArticleDto> {
    let query = article::Entity::find_by_id(article_uuid)
        .one(&state.db)
        .await
        .unwrap()
        .unwrap();

    Json(ArticleDto::from(query))
}

#[utoipa::path(
    get,
    path = "/",
    tag = "article",
    params(ArticleGetParams),
    responses(
        (status=200, body=Vec<ArticleDto>)
    )
)]
pub async fn article_get(
    State(state): State<AppState>,
    Query(params): Query<ArticleGetParams>,
) -> Json<Vec<ArticleDto>> {
    let mut query = article::Entity::find();

    if let Some(feed_uuid) = params.feed_uuid {
        query = query.filter(article::Column::FeedId.eq(feed_uuid));
    }

    let articles = query
        .order_by_desc(article::Column::PublishedAt)
        .select_only()
        .column(article::Column::Id)
        .column(article::Column::Title)
        .column(article::Column::PublishedAt)
        .column(article::Column::FeedId)
        .column(article::Column::Link)
        .column(article::Column::Status)
        .column(article::Column::FetchedAt)
        .column(article::Column::ContentHash)
        .all(&state.db)
        .await
        .unwrap();

    Json(articles.into_iter().map(ArticleDto::from).collect())
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(article_get))
        .routes(routes!(article_get_by_uuid))
}
