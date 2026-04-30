use crate::entities;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryDto {
    pub id: Option<Uuid>,
    pub human_name: String,
    pub human_description: String,
    pub model_description: String
}


impl From<entities::category::Model> for CategoryDto {
    fn from(m: entities::category::Model) -> Self {
        Self {
            id: Some(m.id),
            human_name: m.human_name,
            human_description: m.human_description,
            model_description: m.model_description
        }
    }
}

impl From<CategoryDto> for entities::category::ActiveModel {
    fn from(dto: CategoryDto) -> Self {
        Self {
            id: Set(dto.id.unwrap_or(Uuid::new_v4())),
            human_description: Set(dto.human_description),
            human_name: Set(dto.human_name),
            model_description: Set(dto.model_description)
        }
    }
}
