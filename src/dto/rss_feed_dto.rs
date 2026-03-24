use sea_orm::prelude::Uuid;
use serde::Serialize;
use utoipa::{IntoParams, OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use crate::entities::rss_feed;

#[derive(Serialize, ToSchema)]
pub struct RssFeedDto {
    pub id: Uuid,
    pub url: String,
}


impl From<rss_feed::Model> for RssFeedDto {
    fn from(m: rss_feed::Model) -> Self {
        Self { id: m.id, url: m.url }
    }
}