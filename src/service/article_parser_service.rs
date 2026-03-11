use std::time::Duration;

use anyhow::{Result, anyhow};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use chromiumoxide::browser::BrowserConfig;
use futures::StreamExt;
use legible::parse;
use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
pub struct HtmlArticle {
    pub html_content: String,
    pub title: String,
}

async fn fetch_with_client(client: &Client, url: &str) -> Result<(StatusCode, String)> {
    let response = client.get(url).send().await?;
    let status = response.status();
    let body = response.text().await?;
    Ok((status, body))
}

fn html_looks_blocked(html: &str) -> bool {
    let lower = html.to_lowercase();
    lower.len() < 10_000
}

pub async fn get_url_contents_headless(url: &str) -> Result<String> {
    // --- 1 Normal browser request ---
    eprintln!("[1] Trying classic request");

    let normal_client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120 Safari/537.36")
        .http1_only()
        .timeout(Duration::from_secs(10))
        .build()?;

    if let Ok((status, html)) = fetch_with_client(&normal_client, url).await {
        eprintln!(
            "[1] Status: {} | Length: {} | Blocked: {}",
            status,
            html.len(),
            html_looks_blocked(&html)
        );
        if status.is_success() && !html_looks_blocked(&html) && html.len() > 5000 {
            eprintln!("[1] SUCCESS");
            return Ok(html);
        }
    } else {
        eprintln!("[1] Request failed");
    }

    // --- 2 Googlebot ---
    eprintln!("[2] Trying Googlebot request");

    let mut headers = HeaderMap::new();
    headers.insert(
        "User-Agent",
        HeaderValue::from_static(
            "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
        ),
    );

    let googlebot_client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(10))
        .build()?;

    if let Ok((status, html)) = fetch_with_client(&googlebot_client, url).await {
        eprintln!(
            "[2] Status: {} | Length: {} | Blocked: {}",
            status,
            html.len(),
            html_looks_blocked(&html)
        );
        if status.is_success() && !html_looks_blocked(&html) && html.len() > 5000 {
            eprintln!("[2] SUCCESS");
            return Ok(html);
        }
    } else {
        eprintln!("[2] Request failed");
    }

    // --- 3 AMP fallback ---
    eprintln!("[3] Trying AMP");

    let amp_url = if url.contains('?') {
        format!("{url}&amp=1")
    } else {
        format!("{url}?amp=1")
    };

    if let Ok((status, html)) = fetch_with_client(&normal_client, &amp_url).await {
        eprintln!(
            "[3] Status: {} | Length: {} | Blocked: {}",
            status,
            html.len(),
            html_looks_blocked(&html)
        );
        if status.is_success() && !html_looks_blocked(&html) && html.len() > 3000 {
            eprintln!("[3] SUCCESS");
            return Ok(html);
        }
    } else {
        eprintln!("[3] Request failed");
    }

    // --- 4 Headless ---
    eprintln!("[4] Trying headless browser");

    let html = fetch_with_headless(url).await?;

    eprintln!(
        "[4] Headless result length: {} | Blocked: {}",
        html.len(),
        html_looks_blocked(&html)
    );

    Ok(html)
}

async fn fetch_with_headless(url: &str) -> Result<String> {
    let config = BrowserConfig::builder()
        .chrome_executable("/usr/bin/chromium")
        .build()
        .map_err(|e| anyhow!(e))?;

    let (browser, mut handler) = chromiumoxide::browser::Browser::launch(config).await?;

    tokio::spawn(async move { while let Some(_event) = handler.next().await {} });

    let page = browser.new_page(url).await?;
    tokio::time::sleep(Duration::from_secs(3)).await;

    let html = page.content().await?;

    Ok(html)
}

pub async fn parse_article_from_url(url: &str) -> Result<HtmlArticle> {
    let body = get_url_contents_headless(url).await?;

    let article =
        parse(&body, Some(url), None).map_err(|e| anyhow!("Legible parse error: {}", e))?;

    Ok(HtmlArticle {
        html_content: article.content,
        title: article.title,
    })
}