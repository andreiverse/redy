use crate::api::error::AppError;
use crate::dto::article_dto::ArticleDto;
use crate::dto::feed_dto::{CreateFeedDto, UpdateFeedDto};
use crate::entities::feed;
use crate::service::rss_fetcher_service::rss_fetch;
use crate::{AppState, dto::feed_dto::FeedDto};
use axum::extract::{Path, Query};
use axum::{Json, extract::State};
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, TryIntoModel};
use tower_sessions::Session;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct FeedFilters {
    pub user_id: Option<Uuid>,
}

#[utoipa::path(
    post,
    path = "/",
    tag = "feed",
    request_body = CreateFeedDto,
    responses(
        (status = 200, body = FeedDto)
    )
)]
pub async fn feed_post(
    State(state): State<AppState>,
    session: Session,
    Json(payload): Json<CreateFeedDto>,
) -> Result<Json<FeedDto>, AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;

    if !user.can_create_feeds {
        return Err(AppError::Forbidden("You can't create feeds".to_owned()));
    }

    let feed_ent = feed::ActiveModel {
        id: Set(Uuid::new_v4()),
        url: Set(payload.url),
        name: Set(payload.name),
        default_language: Set(payload.default_language),
        feed_type: Set(payload.feed_type.into()),
        created_at: Set(Utc::now().into()),
        last_fetch: Set(None),
        owner_uuid: Set(Some(user.id))
    };

    let feed = feed_ent
        .insert(&state.db)
        .await
        .expect("Failed to insert feed");

    Ok(Json(FeedDto::from(feed)))
}

#[utoipa::path(
    get,
    path = "/",
    tag = "feed",
    params(
        FeedFilters
    ),
    responses(
        (status=200, body=Vec<FeedDto>)
    )
)]
pub async fn feed_get(
    State(state): State<AppState>,
    session: Session,
    Query(filters): Query<FeedFilters>,
) -> Result<Json<Vec<FeedDto>>, AppError> {
    let mut query = feed::Entity::find();

    if let Some(user_id) = filters.user_id {
        let user = state.auth_service.get_user_from_session(&session).await?;
        if !user.is_admin && user.id != user_id {
            return Err(AppError::Forbidden(
                "You can only view your own feeds".to_owned(),
            ));
        }
        query = query.filter(feed::Column::OwnerUuid.eq(user_id));
    }

    let feeds = query.all(&state.db).await?;

    Ok(Json(feeds.into_iter().map(FeedDto::from).collect()))
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
                rss_fetch(&feed)
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

#[utoipa::path(
    patch,
    path = "/{feed_uuid}",
    tag = "feed",
    request_body = UpdateFeedDto,
    responses(
        (status = 200, body = FeedDto)
    )
)]
pub async fn feed_patch(
    State(state): State<AppState>,
    session: Session,
    Path(feed_uuid): Path<Uuid>,
    Json(payload): Json<UpdateFeedDto>,
) -> Result<Json<FeedDto>, AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    let feed = feed::Entity::find_by_id(feed_uuid)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Feed not found".to_string()))?;

    if feed.owner_uuid != Some(user.id) && !user.is_admin {
        return Err(AppError::Forbidden("You do not own this feed".to_string()));
    }

    let mut active_feed = feed.into_active_model();

    if let Some(url) = payload.url {
        active_feed.url = Set(url);
    }
    if let Some(name) = payload.name {
        active_feed.name = Set(name);
    }
    if let Some(default_language) = payload.default_language {
        active_feed.default_language = Set(default_language);
    }
    if let Some(feed_type) = payload.feed_type {
        active_feed.feed_type = Set(feed_type.into());
    }

    if let Some(owner_uuid) = payload.owner_uuid {
        if !user.is_admin {
            return Err(AppError::Forbidden(
                "Only admins can change the owner of a feed".to_string(),
            ));
        }
        active_feed.owner_uuid = Set(owner_uuid);
    }

    let feed = active_feed.update(&state.db).await?;

    Ok(Json(FeedDto::from(feed)))
}

#[utoipa::path(
    delete,
    path = "/{feed_uuid}",
    tag = "feed",
    responses(
        (status = 200, description = "Feed deleted successfully")
    )
)]
pub async fn feed_delete(
    State(state): State<AppState>,
    session: Session,
    Path(feed_uuid): Path<Uuid>,
) -> Result<(), AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    let feed = feed::Entity::find_by_id(feed_uuid)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Feed not found".to_string()))?;

    if feed.owner_uuid != Some(user.id) && !user.is_admin {
        return Err(AppError::Forbidden("You do not own this feed".to_string()));
    }

    feed::Entity::delete_by_id(feed_uuid)
        .exec(&state.db)
        .await?;

    Ok(())
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(feed_get))
        .routes(routes!(feed_get_by_uuid))
        .routes(routes!(feed_post))
        .routes(routes!(feed_fetch_by_uuid))
        .routes(routes!(feed_patch))
        .routes(routes!(feed_delete))
}
