use anyhow::Context;
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use legible::parse;
use serde::Serialize;

#[derive(Serialize)]
pub struct HtmlArticle {
    pub html_content: String,
    pub title: String,
}

async fn get_url_contents_headless(url: &str) -> Result<String, anyhow::Error> {
    // 1. Configure and Launch Browser (explicit path for Arch)
    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .chrome_executable("/usr/bin/chromium") 
            .build()
            .map_err(anyhow::Error::msg)?
    )
    .await
    .context("Failed to launch chromium instance")?;

    // 2. Spawn the background handler
    let handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    // 3. Navigate to the page
    // Use .map_err(anyhow::Error::msg) to convert String errors
    let page = browser
        .new_page(url)
        .await
        .map_err(anyhow::Error::msg)?;

    // 4. Wait for navigation and extract content
    page.wait_for_navigation()
        .await
        .map_err(anyhow::Error::msg)?;

    let html = page
        .content()
        .await
        .map_err(anyhow::Error::msg)
        .context("Failed to extract page HTML content")?;

    // 5. Safe Cleanup
    // We ignore the error from browser.close() because "oneshot canceled" 
    // simply means the browser closed before it could say goodbye.
    let _ = browser.close().await;
    
    // Ensure the background handler is finished
    handle.abort(); 

    Ok(html)
}

/// Parses the article. Note: Changed return type to anyhow::Result for consistency.
pub async fn parse_article_from_url(url: &str) -> anyhow::Result<HtmlArticle> {
    // Use the headless version we just built
    let body = get_url_contents_headless(url).await?;

    // legible::parse usually returns a Result with a String error or specific error type
    let article = parse(&body, Some(url), None)
        .map_err(|e| anyhow::anyhow!("Legible parse error: {}", e))?;

    Ok(HtmlArticle {
        html_content: article.content,
        title: article.title,
    })
}