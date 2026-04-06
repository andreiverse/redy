use crate::entities::{self, article::Model as ArticleModel};
use crate::entities::sea_orm_active_enums::ArticleStatus;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArticleWithDataDto {
    pub article: ArticleDto,
    pub sentiment_score: Option<f64>
}


#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArticleDto {
    pub id: Uuid,
    pub feed_id: Uuid,
    pub title: String,
    pub feed_description: Option<String>,
    pub link: String,
    pub html_content: Option<String>,
    pub status: ArticleStatusDto,
    pub published_at: Option<DateTime<chrono::FixedOffset>>,
    pub fetched_at: DateTime<chrono::FixedOffset>,
    pub content_hash: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum ArticleStatusDto {
    Pending,
    Extracted,
    ExtractionFailed,
    Done,
}

impl From<ArticleStatus> for ArticleStatusDto {
    fn from(s: ArticleStatus) -> Self {
        match s {
            ArticleStatus::Pending => ArticleStatusDto::Pending,
            ArticleStatus::Extracted => ArticleStatusDto::Extracted,
            ArticleStatus::ExtractionFailed => ArticleStatusDto::ExtractionFailed,
            ArticleStatus::Done => ArticleStatusDto::Done,
        }
    }
}

impl From<ArticleModel> for ArticleDto {
    fn from(m: ArticleModel) -> Self {
        Self {
            id: m.id,
            feed_id: m.feed_id,
            content_hash: m.content_hash,
            title: m.title,
            feed_description: m.feed_description,
            link: m.link,
            html_content: m.html_content,
            status: m.status.into(),
            published_at: m.published_at,
            fetched_at: m.fetched_at,
        }
    }
}

impl From<(ArticleModel, Option<entities::article_data::Model>)> for ArticleWithDataDto {
    fn from(tuple: (ArticleModel, Option<entities::article_data::Model>)) -> Self {
        let (article, article_data_opt) = tuple;
        Self {
            article: ArticleDto::from(article),
            sentiment_score: article_data_opt.and_then(|d| d.sentiment_score) 
        }
    }
}