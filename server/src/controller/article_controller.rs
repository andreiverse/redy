use crate::AppState;
use crate::dto::article_dto::ArticleWithDataDto;
use crate::dto::category_dto::CategoryDto;
use crate::entities::{self, article, category};
use axum::Json;
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait};
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

#[derive(IntoParams, Deserialize)]
pub struct ArticleGetParams {
    pub feed_uuid: Option<Uuid>,
    pub category_id: Option<Uuid>,
}

#[utoipa::path(
    get,
    path = "/categories",
    tag = "article",
    params(ArticleGetParams),
    responses(
        (status=200, body=Vec<CategoryDto>)
    )
)]
pub async fn article_get_categories(
    State(state): State<AppState>,
    Query(params): Query<ArticleGetParams>,
) -> Json<Vec<CategoryDto>> {
    let mut query = category::Entity::find()
        .join(
            sea_orm::JoinType::InnerJoin,
            category::Relation::ArticleData.def(),
        )
        .join(
            sea_orm::JoinType::InnerJoin,
            entities::article_data::Relation::Article.def(),
        )
        .distinct();

    if let Some(feed_uuid) = params.feed_uuid {
        query = query.filter(article::Column::FeedId.eq(feed_uuid));
    }

    let results: Vec<entities::category::Model> = query.all(&state.db).await.unwrap();

    let categories = results
        .into_iter()
        .map(CategoryDto::from)
        .collect();

    Json(categories)
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
        .column(article::Column::HtmlContentFromFeed)
        .limit(50)
        .find_also_related(entities::article_data::Entity);

    if let Some(feed_uuid) = params.feed_uuid {
        query = query.filter(article::Column::FeedId.eq(feed_uuid));
    }

    if let Some(category_id) = params.category_id {
        query = query.filter(entities::article_data::Column::CategoryId.eq(category_id));
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
        .routes(routes!(article_get_categories))
}
