mod api;
mod controller;
mod dto;
mod entities;
mod service;

use axum::Router;
use sea_orm::{Database, DatabaseConnection};
use tracing::Level;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_sessions::{
    Expiry, MemoryStore, SessionManagerLayer,
    cookie::{SameSite, time::Duration},
};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_max_level(Level::INFO)
        .init();
    let db: DatabaseConnection =
        Database::connect("postgres://user:password@localhost:5432/my_app_db").await?;

    let (router, api) = controller::create_controller().split_for_parts();

    let cors = CorsLayer::permissive();

    let state = AppState { db };

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(120)));

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        .merge(router)
        .layer(cors)
        .layer(session_layer)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
