use chrono::{FixedOffset, Utc};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entities::{feed, sea_orm_active_enums::FeedType};

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedDto {
    pub id: Uuid,
    pub last_fetch: Option<chrono::DateTime<FixedOffset>>,
    pub fetched_seconds_ago: Option<f64>,
    pub url: String,
    pub name: String,
    pub default_language: String,
    pub created_at: chrono::DateTime<FixedOffset>,
    pub feed_type: FeedTypeDto,
    pub owner_uuid: Option<Uuid>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum FeedTypeDto {
    Rss,
}

impl From<FeedType> for FeedTypeDto {
    fn from(dto: FeedType) -> Self {
        match dto {
            FeedType::Rss => FeedTypeDto::Rss,
        }
    }
}

impl From<FeedTypeDto> for FeedType {
    fn from(dto: FeedTypeDto) -> Self {
        match dto {
            FeedTypeDto::Rss => FeedType::Rss,
        }
    }
}

impl From<feed::Model> for FeedDto {
    fn from(m: feed::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            url: m.url,
            default_language: m.default_language,
            created_at: m.created_at,
            feed_type: m.feed_type.into(),
            last_fetch: m.last_fetch,
            fetched_seconds_ago: m
                .last_fetch
                .map(|lf| Utc::now().with_timezone(lf.offset()) - lf)
                .map(|d| d.as_seconds_f64()),
            owner_uuid: m.owner_uuid,
        }
    }
}

#[derive(Serialize, ToSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeedDto {
    pub url: String,
    pub name: String,
    pub default_language: String,
    pub feed_type: FeedTypeDto,
}
