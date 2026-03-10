use legible::parse;

pub struct HtmlArticle {
    pub html_content: String,
    pub title: String,
}

pub async fn parse_contents_of_url(url: &str) -> Result<HtmlArticle, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .http1_only()
        .build()?;

    let body = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let article = parse(&body, Some(url), None).map_err(|e| format!("Parse error: {}", e))?;

    Ok(HtmlArticle {
        html_content: article.content,
        title: article.title,
    })
}
