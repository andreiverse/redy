use std::time::Duration;

use async_nats::{jetstream::{self, stream}, service::error};
use tracing::{info, error};
use tokio_stream::StreamExt;

pub async fn scrape_article_worker(jetstream_context: &jetstream::Context) {
    // 1. Define the stream WITH subjects
    let stream = jetstream_context
        .get_or_create_stream(stream::Config {
            name: "SCRAPER".to_string(),
            subjects: vec!["tasks.scrape.>".to_string()],
            ..Default::default()
        })
        .await
        .expect("Should be able to create or get stream");

    // // 2. Create a consumer to read those messages
    let consumer = stream
        .get_or_create_consumer("worker", jetstream::consumer::pull::Config {
            durable_name: Some("worker".to_string()),
            ..Default::default()
        })
        .await
        .expect("Should be able to create consumer");

    let mut messages = consumer.messages().await.unwrap();

    info!("Scraper worker started. Waiting for messages...");

    while let Some(Ok(msg)) = messages.next().await {
        info!("Received: {:?} on {}", String::from_utf8_lossy(&msg.payload), String::from_utf8_lossy(&msg.subject.as_bytes()));
        
        // Use NAK for now so the message stays in the queue for testing
        if let Err(e) = msg.ack_with(jetstream::AckKind::Nak(Some(Duration::from_mins(10)))).await {
            error!("failed to nak: {}", e);
        }
    }
}