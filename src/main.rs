mod api;
mod controller;
mod dto;
mod entities;
mod jobs;
mod service;
use apalis::{
    layers::{WorkerBuilderExt, retry::RetryPolicy},
    prelude::{Monitor, WorkerBuilder},
};
use apalis_board::axum::{
    framework::{ApiBuilder, RegisterRoute},
    ui::ServeUI,
};
use apalis_cron::{CronStream, Schedule, builder::schedule};
use apalis_postgres::PostgresStorage;
use axum::Router;
use chrono::Local;
use sea_orm::{Database, DatabaseConnection, sqlx};
use std::{net::SocketAddr, time::Duration};
use tower::limit::RateLimitLayer;
use tower_http::cors::CorsLayer;
use tower_sessions::{
    Expiry, MemoryStore, SessionManagerLayer,
    cookie::{self, time},
};
use tracing::{Level, info};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::jobs::{
    fetch_articles_job::handle_fetch_articles_job,
    fetch_article_html_job::{FetchArticleHtmlJob, handle_fetch_article_html_job},
};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub fetch_article_html_job_storage: PostgresStorage<FetchArticleHtmlJob>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let database_url = "postgres://user:password@localhost:5432/my_app_db";

    tracing_subscriber::fmt()
        .with_file(false)
        .with_line_number(true)
        .with_max_level(Level::INFO)
        .init();

    let db: DatabaseConnection = Database::connect(database_url).await?;
    let pg_pool = sqlx::PgPool::connect(database_url).await?;
    PostgresStorage::setup(&pg_pool).await?;

    let fetch_article_html_job_storage: PostgresStorage<FetchArticleHtmlJob> =
        PostgresStorage::new(&pg_pool);

    let cors = CorsLayer::permissive();
    let state = AppState {
        db,
        fetch_article_html_job_storage: fetch_article_html_job_storage.clone(),
    };

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::seconds(120)));

    let api_router = controller::create_controller();
    let full_router = OpenApiRouter::new()
        .merge(api_router)
        .layer(session_layer)
        .layer(cors);

    let (router, api) = full_router.split_for_parts();

    let board_api = ApiBuilder::new(Router::new())
        .register(fetch_article_html_job_storage.clone())
        .build();

    let app = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        .with_state(state)
        .nest("/api/v1", board_api)
        .fallback_service(ServeUI::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("Listening on http://{}", addr);

    let worker = Monitor::new()
        .register(move |_| {
            WorkerBuilder::new("fetch-article-html")
                .backend(fetch_article_html_job_storage.clone())
                .layer(RateLimitLayer::new(1, Duration::from_secs(30)))
                .build(handle_fetch_article_html_job)
        })
        .register(move |_| {
            let schedule = schedule().each().minute().build();
            let cron_backend = CronStream::new_with_timezone(schedule, Local);

            WorkerBuilder::new("fetch-articles")
                .backend(cron_backend)
                .retry(RetryPolicy::retries(5))
                .data(pg_pool.clone())
                .build(handle_fetch_articles_job)
        })
        .run();

    let api_task = async {
        axum::serve(listener, app)
            .await
            .map_err(anyhow::Error::from)
    };

    let worker_task = async { worker.await.map_err(anyhow::Error::from) };

    tokio::try_join!(api_task, worker_task)?;

    Ok(())
}
