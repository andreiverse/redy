mod api;
mod controller;
mod dto;
mod entities;
mod service;
mod worker;

use async_nats::jetstream;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing_subscriber::EnvFilter;
use std::{net::SocketAddr, time::Duration};
use tokio::time::interval;
use tower_http::cors::CorsLayer;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer, cookie};
use tracing::{Level, info};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::scrape_article_worker::scrape_article_worker;
use crate::worker::{
    fetch_feeds_worker::fetch_feeds_task,
    scrape_article_worker,
};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_file(false)
        .with_line_number(true)
        .with_env_filter(EnvFilter::new("html5ever=error,info")) 
        .with_max_level(Level::INFO)
        .init();
    let mut opt = ConnectOptions::new("postgres://user:password@localhost:5432/my_app_db");

    opt.sqlx_logging(false); 

    let db = Database::connect(opt).await?;
    let nats_url = "nats://localhost:4222";
    let client = async_nats::connect(nats_url).await?;
    let jetstream = jetstream::new(client);

    let (_router, _api) = controller::create_controller().split_for_parts();

    let cors = CorsLayer::permissive();

    let db_for_worker = db.clone();
    let js_worker = jetstream.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_mins(10));
        info!("Fetching feed articles every 10 minutes...");
        loop {
            ticker.tick().await;
            if let Err(e) = fetch_feeds_task(&db_for_worker, &js_worker).await {
                eprintln!("Fetcher error: {:?}", e);
            }
        }
    });

    let db_for_worker2 = db.clone();
    tokio::spawn(async move {
        scrape_article_worker(&jetstream, &db_for_worker2).await;
    });

    let state = AppState { db };

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(cookie::time::Duration::seconds(120)));
    let api_router = controller::create_controller();

    let full_router = OpenApiRouter::new()
        .merge(api_router)
        .layer(session_layer)
        .layer(cors);

    let (router, api) = full_router.split_for_parts();

    let app = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
