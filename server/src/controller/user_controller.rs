use crate::api::error::AppError;
use crate::dto::user_dto::{UpdateUserDto, UserDto};
use crate::entities::user;
use crate::AppState;
use axum::extract::{Path, State};
use axum::Json;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, QueryOrder, Set};
use tower_sessions::Session;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/",
    tag = "user",
    responses(
        (status = 200, body = Vec<UserDto>)
    )
)]
pub async fn user_get_all(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<Vec<UserDto>>, AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    if !user.is_admin {
        return Err(AppError::Forbidden("Only admins can list users".to_owned()));
    }

    let users = user::Entity::find().order_by_asc(user::Column::Id).all(&state.db).await?;
    Ok(Json(users.into_iter().map(UserDto::from).collect()))
}

#[utoipa::path(
    get,
    path = "/{user_id}",
    tag = "user",
    responses(
        (status = 200, body = UserDto)
    )
)]
pub async fn user_get_by_id(
    State(state): State<AppState>,
    session: Session,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserDto>, AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    if !user.is_admin {
        return Err(AppError::Forbidden("Only admins can view users".to_owned()));
    }

    let target_user = user::Entity::find_by_id(user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok(Json(UserDto::from(target_user)))
}

#[utoipa::path(
    patch,
    path = "/{user_id}",
    tag = "user",
    request_body = UpdateUserDto,
    responses(
        (status = 200, body = UserDto)
    )
)]
pub async fn user_patch(
    State(state): State<AppState>,
    session: Session,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserDto>,
) -> Result<Json<UserDto>, AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    if !user.is_admin {
        return Err(AppError::Forbidden("Only admins can update users".to_owned()));
    }

    let target_user = user::Entity::find_by_id(user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    let mut active_user = target_user.into_active_model();

    if let Some(is_admin) = payload.is_admin {
        active_user.is_admin = Set(is_admin);
    }
    if let Some(can_create_feeds) = payload.can_create_feeds {
        active_user.can_create_feeds = Set(can_create_feeds);
    }

    let updated_user = active_user.update(&state.db).await?;

    Ok(Json(UserDto::from(updated_user)))
}

#[utoipa::path(
    delete,
    path = "/{user_id}",
    tag = "user",
    responses(
        (status = 200, description = "User deleted successfully")
    )
)]
pub async fn user_delete(
    State(state): State<AppState>,
    session: Session,
    Path(user_id): Path<Uuid>,
) -> Result<(), AppError> {
    let user = state.auth_service.get_user_from_session(&session).await?;
    if !user.is_admin {
        return Err(AppError::Forbidden("Only admins can delete users".to_owned()));
    }

    user::Entity::delete_by_id(user_id)
        .exec(&state.db)
        .await?;

    Ok(())
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(user_get_all))
        .routes(routes!(user_get_by_id))
        .routes(routes!(user_patch))
        .routes(routes!(user_delete))
}
