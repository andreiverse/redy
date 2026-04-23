use crate::entities::article::ActiveModel as ArticleActiveModel;
use crate::entities::feed;
use crate::entities::sea_orm_active_enums::ArticleStatus;
use reqwest::Client;
use sea_orm::Set;
use sha2::{Digest, Sha256};
use std::time::Duration;

fn hash_link(link: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(link.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

pub async fn rss_fetch(feed: &feed::Model) -> anyhow::Result<Vec<ArticleActiveModel>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120 Safari/537.36")
        .http1_only()
        .timeout(Duration::from_secs(10))
        .build()?;

    let body = client.get(feed.url.clone()).send().await?.text().await?;
    let channel = rss::Channel::read_from(body.as_bytes())?;
    let now = chrono::Utc::now().fixed_offset();

    let articles = channel
        .items()
        .iter()
        .map(|item| {
            let link = item.link().unwrap_or("").to_string();
            let published_at = item.pub_date().and_then(|d| {
                chrono::DateTime::parse_from_rfc3339(d)
                    .or_else(|_| chrono::DateTime::parse_from_rfc2822(d))
                    .ok()
            });
            let language = channel.language().unwrap_or(&feed.default_language);

            ArticleActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                feed_id: Set(feed.id),
                title: Set(item.title().unwrap_or("").to_string()),
                feed_description: Set(item.description().map(str::to_string)),
                link: Set(link.clone()),
                content_hash: Set(hash_link(&link)),
                html_content: Set(item.content().map(|f| f.to_owned())),
                status: Set(ArticleStatus::Pending),
                published_at: Set(published_at),
                fetched_at: Set(now),
                language: Set(language.to_owned()),
            }
        })
        .collect();

    Ok(articles)
}
