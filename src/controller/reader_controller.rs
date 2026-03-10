use axum::{extract::Query, response::{Html, Response, IntoResponse}};
use serde::Deserialize;
use crate::{api::error::AppError, service::reader_service::parse_contents_of_url};

#[derive(Deserialize)]
pub struct ReaderGetParams {
    url: String,
}

pub async fn reader_get(Query(params): Query<ReaderGetParams>) -> Response {
    match parse_contents_of_url(&params.url).await {
        Ok(html_content) => Html(html_content.html_content).into_response(),

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

