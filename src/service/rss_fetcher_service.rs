use std::time::Duration;

use reqwest::Client;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, ToSchema, Serialize)]
pub struct RssNews {
    pub title: String,
    pub description: Option<String>,
    pub link: String,
    pub author: Option<String>,
    pub guid: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub published_at_raw: Option<String>,
}

pub async fn rss_fetch(url: String) -> anyhow::Result<Vec<RssNews>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120 Safari/537.36")
        .http1_only()
        .timeout(Duration::from_secs(10))
        .build()?;

    let response = client.get(url).send().await?;
    let body = response.text().await?;

    let channel = rss::Channel::read_from(body.as_bytes())?;

    let news = channel
        .items()
        .iter()
        .map(|item| RssNews {
            published_at_raw: item.pub_date().map(str::to_string),
            published_at: item
                .pub_date()
                .and_then(|d| {
                    chrono::DateTime::parse_from_rfc3339(d)
                        .or_else(|_| chrono::DateTime::parse_from_rfc2822(d))
                        .ok()
                })
                .map(|d| d.with_timezone(&chrono::Utc)),
            guid: item.guid().map(|g| g.value().to_string()),
            title: item.title().unwrap_or("").to_string(),
            description: item.description().map(str::to_string),
            author: item.author().map(str::to_string),
            link: item.link().unwrap_or("").to_string(),
        })
        .collect();

    Ok(news)
}
