use anyhow::{Result, anyhow};
use chromiumoxide::browser::BrowserConfig;
use futures::StreamExt;
use std::time::Duration;
use super::html_looks_blocked;

pub async fn fetch(url: &str) -> Result<String> {
    eprintln!("[4] Trying headless browser");
    let config = BrowserConfig::builder()
        .chrome_executable("/usr/bin/chromium")
        .build()
        .map_err(|e| anyhow!(e))?;

    let (browser, mut handler) = chromiumoxide::browser::Browser::launch(config).await?;

    tokio::spawn(async move { while let Some(_event) = handler.next().await {} });

    let page = browser.new_page(url).await?;
    tokio::time::sleep(Duration::from_secs(3)).await;

    let html = page.content().await?;

    eprintln!(
        "[4] Headless result length: {} | Blocked: {}",
        html.len(),
        html_looks_blocked(&html)
    );

    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    async fn start_mock_server() -> (SocketAddr, tokio::task::JoinHandle<()>) {
        let app = Router::new()
            .route("/", get(|| async { "<html><body>Headless Test</body></html>" }));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (addr, handle)
    }

    #[tokio::test]
    async fn test_headless_fetch() {
        let (addr, handle) = start_mock_server().await;
        let url = format!("http://{}", addr);
        let result = fetch(&url).await;
        assert!(result.is_ok());
        handle.abort();
    }
}
