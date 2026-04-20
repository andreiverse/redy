use crate::AppState;
use crate::dto::article_dto::ArticleWithDataDto;
use crate::entities::{self, article};
use axum::Json;
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};
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
        (status=200, body=ArticleWithDataDto)
    )
)]
pub async fn article_get_by_uuid(
    State(state): State<AppState>,
    Path(article_uuid): Path<Uuid>,
) -> Json<ArticleWithDataDto> {
    let query = article::Entity::find()
        .filter(article::Column::Id.eq(article_uuid))
        .find_also_related(entities::article_data::Entity)
        .one(&state.db)
        .await
        .unwrap()
        .unwrap(); // (ArticleModel, Option<ArticleDataModel>)

    Json(ArticleWithDataDto::from(query))
}

#[utoipa::path(
    get,
    path = "/",
    tag = "article",
    params(ArticleGetParams),
    responses(
        (status=200, body=Vec<ArticleWithDataDto>)
    )
)]
pub async fn article_get(
    State(state): State<AppState>,
    Query(params): Query<ArticleGetParams>,
) -> Json<Vec<ArticleWithDataDto>> {
    let mut query = article::Entity::find()
        .filter(article::Column::HtmlContent.is_not_null())
        .select_only()
        .column(article::Column::Id)
        .column(article::Column::Title)
        .column(article::Column::PublishedAt)
        .column(article::Column::FeedId)
        .column(article::Column::Link)
        .column(article::Column::Status)
        .column(article::Column::Language)
        .column(article::Column::FetchedAt)
        .column(article::Column::ContentHash)
        .limit(50)
        .find_also_related(entities::article_data::Entity);

    if let Some(feed_uuid) = params.feed_uuid {
        query = query.filter(article::Column::FeedId.eq(feed_uuid));
    }

    let results = query
        .order_by_desc(article::Column::PublishedAt)
        .all(&state.db)
        .await
        .unwrap(); // Vec<(ArticleModel, Option<ArticleDataModel>)>

    Json(results.into_iter().map(ArticleWithDataDto::from).collect())
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(article_get))
        .routes(routes!(article_get_by_uuid))
}
