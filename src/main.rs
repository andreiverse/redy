mod api;
mod controller;
mod entities;
mod service;
mod dto;

use axum::{Router, routing::get};
use sea_orm::{Database, DatabaseConnection};
use utoipa_swagger_ui::SwaggerUi;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

use crate::controller::reader_controller::reader_get;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let db: DatabaseConnection =
        Database::connect("postgres://user:password@localhost:5432/my_app_db").await?;

    let (router, api) = controller::create_controller().split_for_parts();

    let cors = CorsLayer::permissive();

    let state = AppState { db };

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        .merge(router)
        .layer(cors)
        
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
