use crate::{
    AppState,
    api::error::AppError,
    service::article_parser_service::{HtmlArticle, parse_article_from_url},
};
use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use tower_sessions::Session;
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};

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
pub async fn reader_get(
    State(state): State<AppState>,
    session: Session,
    Query(params): Query<ReaderGetParams>
) -> Response {
    let user = match state.auth_service.get_user_from_session(&session).await {
        Ok(user) => user,
        Err(e) => return e.into_response(),
    };

    if !user.is_admin {
        return AppError::Forbidden("Only admins can use the reader".to_owned()).into_response();
    }

    match parse_article_from_url(&params.url).await {
        Ok(html_content) => Json(html_content).into_response(),

        Err(e) => {
            let report = format!("Error: {:?}", e);
            eprintln!("{}", report);
            AppError::Internal(report).into_response()
        }
    }
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(reader_get))
}
