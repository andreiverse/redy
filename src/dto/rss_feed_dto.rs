use sea_orm::prelude::Uuid;
use serde::Serialize;

use crate::entities::rss_feed;

#[derive(Serialize)]
pub struct RssFeedDto {
    pub id: Uuid,
    pub url: String,
}

impl From<rss_feed::Model> for RssFeedDto {
    fn from(m: rss_feed::Model) -> Self {
        Self { id: m.id, url: m.url }
    }
}