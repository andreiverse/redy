use anyhow::{Result, anyhow};
use legible::parse;
use serde::Serialize;
use utoipa::ToSchema;

use crate::service::article_fetcher::{normal, googlebot, amp, headless};

#[derive(Serialize, ToSchema)]
pub struct HtmlArticle {
    pub html_content: String,
    pub title: String,
}

pub async fn get_url_contents_headless(url: &str) -> Result<String> {
    if let Ok(html) = normal::fetch(url).await {
        eprintln!("[1] SUCCESS");
        return Ok(html);
    }

    if let Ok(html) = googlebot::fetch(url).await {
        eprintln!("[2] SUCCESS");
        return Ok(html);
    }

    if let Ok(html) = amp::fetch(url).await {
        eprintln!("[3] SUCCESS");
        return Ok(html);
    }

    let html = headless::fetch(url).await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    async fn start_mock_server() -> (SocketAddr, tokio::task::JoinHandle<()>) {
        let app = Router::new()
            .route("/", get(|| async { "<html><head><title>Test Title</title></head><body>" .to_string() + &"a".repeat(11000) + "</body></html>" }));

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

        let result = parse_article_from_url(&url).await;
        assert!(result.is_ok());
        let article = result.unwrap();
        assert_eq!(article.title, "Test Title");
        assert!(article.html_content.len() > 0);

        handle.abort();
    }
}
