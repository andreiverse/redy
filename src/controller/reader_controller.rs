use axum::{Json, extract::Query, response::{Html, IntoResponse, Response}};
use serde::Deserialize;
use crate::{api::error::AppError, service::article_parser_service::parse_article_from_url};

#[derive(Deserialize)]
pub struct ReaderGetParams {
    url: String,
}

pub async fn reader_get(Query(params): Query<ReaderGetParams>) -> Response {
    match parse_article_from_url(&params.url).await {
        Ok(html_content) => Json(html_content).into_response(),

        Err(e) => {
            let mut report = format!("Error: {}", e);
            let mut source = e.source();

            while let Some(cause) = source {
                report.push_str(&format!("\nCaused by: {}", cause));
                source = cause.source();
            }

            eprintln!("Error: {}", report);

           AppError::Internal.into_response() 
        }
    }
}

