use super::{fetch_with_client, html_looks_blocked, is_content_acceptable};
use anyhow::{Result, anyhow};
use reqwest::Client;
use std::time::Duration;
use tracing::debug;

pub async fn fetch(url: &str) -> Result<String> {
    debug!("[3] Trying AMP");
    let amp_url = if url.contains('?') {
        format!("{url}&amp=1")
    } else {
        format!("{url}?amp=1")
    };

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120 Safari/537.36")
        .http1_only()
        .timeout(Duration::from_secs(10))
        .build()?;

    let (status, html) = fetch_with_client(&client, &amp_url).await?;
    debug!(
        "[3] Status: {} | Length: {} | Blocked: {}",
        status,
        html.len(),
        html_looks_blocked(&html)
    );

    if is_content_acceptable(status, &html, 3000) {
        return Ok(html);
    }
    Err(anyhow!("[3] Content not acceptable"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get};
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    async fn start_mock_server() -> (SocketAddr, tokio::task::JoinHandle<()>) {
        let app = Router::new().route(
            "/",
            get(|| async { "<html><body>".to_string() + &"a".repeat(11000) + "</body></html>" }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (addr, handle)
    }

    #[tokio::test]
    async fn test_amp_fetch() {
        let (addr, handle) = start_mock_server().await;
        let url = format!("http://{}", addr);
        let result = fetch(&url).await;
        assert!(result.is_ok());
        handle.abort();
    }
}
