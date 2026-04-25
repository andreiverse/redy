use axum::{
    extract::MatchedPath,
    http::Request,
    middleware::Next,
    response::IntoResponse,
};
use std::time::Instant;

pub async fn track_metrics(req: Request<axum::body::Body>, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|matched_path| matched_path.as_str().to_owned())
        .unwrap_or_else(|| req.uri().path().to_owned());
    let method = req.method().to_string();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    metrics::counter!(
        "http_requests_total",
        "method" => method.clone(),
        "path" => path.clone(),
        "status" => status
    ).increment(1);

    metrics::histogram!(
        "http_request_duration_seconds",
        "method" => method,
        "path" => path
    ).record(latency);

    response
}
