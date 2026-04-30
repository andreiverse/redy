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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedCategoryDto {
    pub feed_id: Uuid,
    pub category_id: Uuid,
    pub model_description_override: Option<String>,
}

impl From<entities::feed_category::Model> for FeedCategoryDto {
    fn from(m: entities::feed_category::Model) -> Self {
        Self {
            feed_id: m.feed_id,
            category_id: m.category_id,
            model_description_override: m.model_description_override,
        }
    }
}

impl From<FeedCategoryDto> for entities::feed_category::ActiveModel {
    fn from(dto: FeedCategoryDto) -> Self {
        Self {
            feed_id: Set(dto.feed_id),
            category_id: Set(dto.category_id),
            model_description_override: Set(dto.model_description_override),
        }
    }
}
