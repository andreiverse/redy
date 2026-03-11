mod api;
mod controller;
mod entities;
mod service;
mod dto;

use axum::{Router, routing::get};
use sea_orm::{Database, DatabaseConnection};
use std::net::SocketAddr;

use crate::controller::{reader_controller::reader_get, rss_feed_controller::{rss_feed_get, rss_feed_get_by_uuid}};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let db: DatabaseConnection =
        Database::connect("postgres://user:password@localhost:5432/my_app_db").await?;

    let state = AppState { db };

    let app = Router::new()
        .route("/reader", get(reader_get))
        .route("/rss_feed", get(rss_feed_get))
        .route("/rss_feed/:rss_feed_uuid", get(rss_feed_get_by_uuid))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
