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
}

impl From<user::Model> for UserDto {
    fn from(m: user::Model) -> Self {
        Self {
            id: m.id,
            email: m.email,
            username: m.username,
        }
    }
}
