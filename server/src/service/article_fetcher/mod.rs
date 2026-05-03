use anyhow::Result;
use axum::http::StatusCode;
use reqwest::Client;

pub mod amp;
pub mod googlebot;
pub mod normal;

pub async fn fetch_with_client(client: &Client, url: &str) -> Result<(StatusCode, String)> {
    let response = client.get(url).send().await?;
    let status = response.status();
    let body = response.text().await?;
    Ok((status, body))
}

pub fn html_looks_blocked(html: &str) -> bool {
    let lower = html.to_lowercase();
    lower.len() < 10_000 || lower.contains("please enable javascript and cookies in your browser")
}

pub fn is_content_acceptable(status: StatusCode, html: &str, min_len: usize) -> bool {
    status.is_success() && !html_looks_blocked(html) && html.len() > min_len
}
