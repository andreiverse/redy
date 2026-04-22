use crate::api::error::AppError;
use crate::dto::article_dto::ArticleDto;
use crate::dto::feed_dto::CreateFeedDto;
use crate::entities::feed;
use crate::service::rss_fetcher_service::rss_fetch;
use crate::{AppState, dto::feed_dto::FeedDto};
use axum::extract::Path;
use axum::{Json, extract::State};
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, EntityTrait, TryIntoModel};
use tower_sessions::Session;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

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

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(feed_get))
        .routes(routes!(feed_get_by_uuid))
        .routes(routes!(feed_post))
        .routes(routes!(feed_fetch_by_uuid))
}
