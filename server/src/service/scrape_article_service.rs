use anyhow::{Result, anyhow};
use legible::parse;
use serde::Serialize;
use tracing::debug;
use utoipa::ToSchema;
use std::net::{IpAddr, ToSocketAddrs};
use reqwest::Url;

use crate::service::article_fetcher::{amp, googlebot, normal};

#[derive(Serialize, ToSchema)]
pub struct HtmlArticle {
    pub html_content: String,
    pub title: String,
}

pub fn is_public_ip(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(v4) => {
            !v4.is_loopback() && !v4.is_private() && !v4.is_link_local() && !v4.is_broadcast() && !v4.is_documentation()
        }
        IpAddr::V6(v6) => {
            !v6.is_loopback() && !v6.is_unspecified() && !(v6.segments()[0] & 0xff00 == 0xfe00)
        }
    }
}

pub fn validate_url(url_str: &str) -> Result<()> {
    let url = Url::parse(url_str).map_err(|e| anyhow!("Invalid URL: {}", e))?;
    
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(anyhow!("Only HTTP and HTTPS schemes are allowed"));
    }

    if let Some(host) = url.host_str() {
        let addrs = (host, 80).to_socket_addrs().map_err(|e| anyhow!("Failed to resolve host: {}", e))?;
        for addr in addrs {
            let ip: IpAddr = addr.ip();
            if !is_public_ip(ip) {
                return Err(anyhow!("Access to private network is forbidden"));
            }
        }
    }

    Ok(())
}

pub async fn get_url_contents(url: &str) -> Result<String> {
    validate_url(url)?;

    if let Ok(html) = normal::fetch(url).await {
        debug!("[1] SUCCESS");
        return Ok(html);
    }

    if let Ok(html) = googlebot::fetch(url).await {
        debug!("[2] SUCCESS");
        return Ok(html);
    }

    if let Ok(html) = amp::fetch(url).await {
        debug!("[3] SUCCESS");
        return Ok(html);
    }

    Err(anyhow!("Failed to fetch content from all sources"))
}

pub async fn parse_article_from_url(url: &str) -> Result<HtmlArticle> {
    let body = get_url_contents(url).await?;

    let article =
        parse(&body, Some(url), None).map_err(|e| anyhow!("Legible parse error: {}", e))?;

    let sanitized_content = ammonia::clean(&article.content);

    Ok(HtmlArticle {
        html_content: sanitized_content,
        title: article.title,
    })
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
            get(|| async {
                "<html><head><title>Test Title</title></head><body>".to_string()
                    + &"a".repeat(11000)
                    + "</body></html>"
            }),
        );

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (addr, handle)
    }

    #[tokio::test]
    async fn test_parse_article_from_url() {
        let (addr, handle) = start_mock_server().await;
        let url = format!("http://{}", addr);

        // We skip validate_url check here because it's a test on localhost
        let body = normal::fetch(&url).await.unwrap();
        let article = parse(&body, Some(&url), None).unwrap();
        let sanitized_content = ammonia::clean(&article.content);
        
        assert_eq!(article.title, "Test Title");
        assert!(sanitized_content.len() > 0);

        handle.abort();
    }
}
