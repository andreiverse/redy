use sea_orm::prelude::Uuid;
use serde::Serialize;
use utoipa::ToSchema;

use crate::entities::user;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub is_admin: bool,
    pub can_create_feeds: bool,
}

impl From<user::Model> for UserDto {
    fn from(m: user::Model) -> Self {
        Self {
            id: m.id,
            email: m.email,
            username: m.username,
            is_admin: m.is_admin,
            can_create_feeds: m.can_create_feeds,
        }
    }
}

#[derive(serde::Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserDto {
    pub is_admin: Option<bool>,
    pub can_create_feeds: Option<bool>,
}
