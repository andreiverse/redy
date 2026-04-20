use super::{fetch_with_client, html_looks_blocked, is_content_acceptable};
use anyhow::{Result, anyhow};
use axum::http::{HeaderMap, HeaderValue};
use reqwest::Client;
use std::time::Duration;

pub async fn fetch(url: &str) -> Result<String> {
    eprintln!("[2] Trying Googlebot request");
    let mut headers = HeaderMap::new();
    headers.insert(
        "User-Agent",
        HeaderValue::from_static(
            "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
        ),
    );

    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(10))
        .build()?;

    let (status, html) = fetch_with_client(&client, url).await?;
    eprintln!(
        "[2] Status: {} | Length: {} | Blocked: {}",
        status,
        html.len(),
        html_looks_blocked(&html)
    );

    if is_content_acceptable(status, &html, 5000) {
        return Ok(html);
    }
    Err(anyhow!("[2] Content not acceptable"))
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
    async fn test_googlebot_fetch() {
        let (addr, handle) = start_mock_server().await;
        let url = format!("http://{}", addr);
        let result = fetch(&url).await;
        assert!(result.is_ok());
        handle.abort();
    }
}
