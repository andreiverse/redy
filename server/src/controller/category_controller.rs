use crate::api::error::AppError;
use crate::dto::category_dto::CategoryDto;
use crate::entities::category;
use crate::AppState;
use axum::extract::{Path, State};
use axum::Json;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, QueryOrder};
use tower_sessions::Session;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/",
    tag = "category",
    responses(
        (status = 200, body = Vec<CategoryDto>)
    )
)]
pub async fn category_get_all(
    State(state): State<AppState>,
) -> Result<Json<Vec<CategoryDto>>, AppError> {
    let categories = category::Entity::find()
        .order_by_asc(category::Column::HumanName)
        .all(&state.db)
        .await?;
    Ok(Json(categories.into_iter().map(CategoryDto::from).collect()))
}

#[utoipa::path(
    post,
    path = "/",
    tag = "category",
    request_body = CategoryDto,
    responses(
        (status = 200, body = CategoryDto)
    )
)]
pub async fn category_post(
    State(state): State<AppState>,
    session: Session,
    Json(payload): Json<CategoryDto>,
) -> Result<Json<CategoryDto>, AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    if !user.is_admin {
        return Err(AppError::Forbidden("Only admins can create categories".to_owned()));
    }

    let category_active: category::ActiveModel = payload.into();
    let category = category_active.insert(&state.db).await?;

    Ok(Json(CategoryDto::from(category)))
}

#[utoipa::path(
    get,
    path = "/{id}",
    tag = "category",
    responses(
        (status = 200, body = CategoryDto),
        (status = 404, description = "Category not found")
    )
)]
pub async fn category_get_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CategoryDto>, AppError> {
    let category = category::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Category not found".to_string()))?;

    Ok(Json(CategoryDto::from(category)))
}

#[utoipa::path(
    put,
    path = "/{id}",
    tag = "category",
    request_body = CategoryDto,
    responses(
        (status = 200, body = CategoryDto),
        (status = 404, description = "Category not found")
    )
)]
pub async fn category_put(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<Uuid>,
    Json(payload): Json<CategoryDto>,
) -> Result<Json<CategoryDto>, AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    if !user.is_admin {
        return Err(AppError::Forbidden("Only admins can update categories".to_owned()));
    }

    let category = category::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Category not found".to_string()))?;

    let mut active_model = category.into_active_model();
    active_model.human_name = sea_orm::ActiveValue::Set(payload.human_name);
    active_model.human_description = sea_orm::ActiveValue::Set(payload.human_description);
    active_model.model_description = sea_orm::ActiveValue::Set(payload.model_description);

    let category = active_model.update(&state.db).await?;

    Ok(Json(CategoryDto::from(category)))
}

#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "category",
    responses(
        (status = 200, description = "Category deleted successfully"),
        (status = 404, description = "Category not found")
    )
)]
pub async fn category_delete(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    if !user.is_admin {
        return Err(AppError::Forbidden("Only admins can delete categories".to_owned()));
    }

    let result = category::Entity::delete_by_id(id).exec(&state.db).await?;
    if result.rows_affected == 0 {
        return Err(AppError::NotFound("Category not found".to_string()));
    }

    Ok(())
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(category_get_all))
        .routes(routes!(category_post))
        .routes(routes!(category_get_by_id))
        .routes(routes!(category_put))
        .routes(routes!(category_delete))
}
