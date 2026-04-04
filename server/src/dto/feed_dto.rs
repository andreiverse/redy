use chrono::FixedOffset;
use sea_orm::{ActiveEnum, prelude::Uuid};
use serde::Serialize;
use utoipa::ToSchema;

use crate::entities::feed;

#[derive(Serialize, ToSchema)]
pub struct FeedDto {
    pub id: Uuid,
    pub url: String,
    pub created_at: chrono::DateTime<FixedOffset>,
    pub feed_type: String,
}

impl From<feed::Model> for FeedDto {
    fn from(m: feed::Model) -> Self {
        Self {
            id: m.id,
            url: m.url,
            created_at: m.created_at,
            feed_type: m.feed_type.to_value(),
        }
    }
}
