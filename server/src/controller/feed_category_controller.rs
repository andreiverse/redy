use crate::api::error::AppError;
use crate::dto::category_dto::FeedCategoryDto;
use crate::entities::{article, article_data, category, feed, feed_category};
use crate::AppState;
use axum::extract::{Path, State};
use axum::Json;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder,
    QuerySelect, QueryTrait, RelationTrait,
};
use tower_sessions::Session;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/{feed_id}/categories",
    tag = "feed_category",
    responses(
        (status = 200, body = Vec<FeedCategoryDto>)
    )
)]
pub async fn feed_category_get_all(
    State(state): State<AppState>,
    Path(feed_id): Path<Uuid>,
) -> Result<Json<Vec<FeedCategoryDto>>, AppError> {
    let feed_categories = feed_category::Entity::find()
        .filter(feed_category::Column::FeedId.eq(feed_id))
        .join(
            sea_orm::JoinType::InnerJoin,
            feed_category::Relation::Category.def(),
        )
        .order_by_asc(category::Column::HumanName)
        .all(&state.db)
        .await?;
    Ok(Json(feed_categories.into_iter().map(FeedCategoryDto::from).collect()))
}

#[utoipa::path(
    post,
    path = "/{feed_id}/categories",
    tag = "feed_category",
    request_body = FeedCategoryDto,
    responses(
        (status = 200, body = FeedCategoryDto)
    )
)]
pub async fn feed_category_post(
    State(state): State<AppState>,
    session: Session,
    Path(feed_id): Path<Uuid>,
    Json(mut payload): Json<FeedCategoryDto>,
) -> Result<Json<FeedCategoryDto>, AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    let feed = feed::Entity::find_by_id(feed_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Feed not found".to_string()))?;

    if feed.owner_uuid != Some(user.id) && !user.is_admin {
        return Err(AppError::Forbidden("You do not own this feed".to_string()));
    }

    payload.feed_id = feed_id; // Ensure feed_id from path is used
    let feed_category_active: feed_category::ActiveModel = payload.into();
    let feed_category = feed_category_active.insert(&state.db).await?;

    Ok(Json(FeedCategoryDto::from(feed_category)))
}

#[utoipa::path(
    put,
    path = "/{feed_id}/categories/{category_id}",
    tag = "feed_category",
    request_body = FeedCategoryDto,
    responses(
        (status = 200, body = FeedCategoryDto),
        (status = 404, description = "Feed category not found")
    )
)]
pub async fn feed_category_put(
    State(state): State<AppState>,
    session: Session,
    Path((feed_id, category_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<FeedCategoryDto>,
) -> Result<Json<FeedCategoryDto>, AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    let feed = feed::Entity::find_by_id(feed_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Feed not found".to_string()))?;

    if feed.owner_uuid != Some(user.id) && !user.is_admin {
        return Err(AppError::Forbidden("You do not own this feed".to_string()));
    }

    let feed_category = feed_category::Entity::find_by_id((feed_id, category_id))
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Feed category not found".to_string()))?;

    let mut active_model = feed_category.into_active_model();
    active_model.model_description_override = sea_orm::ActiveValue::Set(payload.model_description_override);

    let feed_category = active_model.update(&state.db).await?;

    Ok(Json(FeedCategoryDto::from(feed_category)))
}

#[utoipa::path(
    delete,
    path = "/{feed_id}/categories/{category_id}",
    tag = "feed_category",
    responses(
        (status = 200, description = "Feed category deleted successfully"),
        (status = 404, description = "Feed category not found")
    )
)]
pub async fn feed_category_delete(
    State(state): State<AppState>,
    session: Session,
    Path((feed_id, category_id)): Path<(Uuid, Uuid)>,
) -> Result<(), AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    let feed = feed::Entity::find_by_id(feed_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Feed not found".to_string()))?;

    if feed.owner_uuid != Some(user.id) && !user.is_admin {
        return Err(AppError::Forbidden("You do not own this feed".to_string()));
    }

    let result = feed_category::Entity::delete_by_id((feed_id, category_id))
        .exec(&state.db)
        .await?;
    if result.rows_affected == 0 {
        return Err(AppError::NotFound("Feed category not found".to_string()));
    }

    article_data::Entity::update_many()
        .col_expr(
            article_data::Column::CategoryId,
            sea_orm::sea_query::Expr::value(sea_orm::Value::Uuid(None)),
        )
        .filter(
            article_data::Column::Id.in_subquery(
                article::Entity::find()
                    .select_only()
                    .column(article::Column::Id)
                    .filter(article::Column::FeedId.eq(feed_id))
                    .into_query(),
            ),
        )
        .filter(article_data::Column::CategoryId.eq(category_id))
        .exec(&state.db)
        .await?;

    Ok(())
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(feed_category_get_all))
        .routes(routes!(feed_category_post))
        .routes(routes!(feed_category_put))
        .routes(routes!(feed_category_delete))
}
