use axum::{Json, extract::Query, response::{IntoResponse, Response}};
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};
use crate::{AppState, api::error::AppError, service::article_parser_service::{HtmlArticle, parse_article_from_url}};

#[derive(Deserialize, IntoParams)]
pub struct ReaderGetParams {
    url: String,
}

#[utoipa::path(
    get,
    path = "/",
    params(ReaderGetParams),
    tag = "reader",
    responses(
        (status=200, body=HtmlArticle)
    )
)]
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


pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(reader_get))
}

