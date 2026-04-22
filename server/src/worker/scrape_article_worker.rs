use std::time::Duration;

use async_nats::jetstream::{self, stream};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel,
};
use tokio_stream::StreamExt;
use tracing::{error, info};
use uuid::Uuid;

use crate::{entities::article, service::article_parser_service};

pub enum HandleResult {
    Success,
    NotFound,
}

pub async fn handle_article(
    db: &DatabaseConnection,
    handle_uuid: Uuid,
) -> Result<HandleResult, anyhow::Error> {
    let article = match article::Entity::find_by_id(handle_uuid).one(db).await? {
        Some(a) => a,
        None => return Ok(HandleResult::NotFound),
    };

    let article_link = article.link.clone();
    let mut active_article = article.into_active_model();

    let html_content = article_parser_service::parse_article_from_url(&article_link).await?;

    active_article.html_content = Set(Some(html_content.html_content));
    active_article.update(db).await?;

    Ok(HandleResult::Success)
}

pub async fn scrape_article_worker(
    jetstream_context: &jetstream::Context,
    db: &DatabaseConnection,
) {
    let stream = jetstream_context
        .get_or_create_stream(stream::Config {
            name: "SCRAPER".to_string(),
            subjects: vec!["tasks.scrape.>".to_string()],
            ..Default::default()
        })
        .await
        .expect("Should be able to create or get stream");

    let consumer = stream
        .get_or_create_consumer(
            "worker",
            jetstream::consumer::pull::Config {
                durable_name: Some("worker".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("Should be able to create consumer");

    let mut messages = consumer.messages().await.unwrap();

    info!("Scraper worker started. Waiting for messages...");

    while let Some(Ok(msg)) = messages.next().await {
        let article_uuid = match Uuid::from_slice(&msg.payload) {
            Ok(uuid) => uuid,
            Err(e) => {
                error!("Invalid UUID bytes (len={}): {}", msg.payload.len(), e);
                if let Err(e) = msg.ack().await {
                    error!("Failed to ack invalid message: {}", e);
                }
                continue; // skip processing
            }
        };

        info!(
            "Received: {:?} on {}",
            article_uuid,
            String::from_utf8_lossy(msg.subject.as_bytes())
        );

        match handle_article(db, article_uuid).await {
            Ok(HandleResult::Success) => {
                if let Err(e) = jetstream_context
                    .publish("tasks.ml.sentimental-analysis", msg.payload.clone())
                    .await
                {
                    error!(
                        "Couldn't publish task tasks.ml.sentimental-analysis, naking with 30 minutes: {}",
                        e
                    );

                    msg.ack_with(jetstream::AckKind::Nak(Some(Duration::from_secs(30 * 60))))
                        .await
                        .ok();
                } else {
                    msg.ack().await.ok();
                }
            }

            Ok(HandleResult::NotFound) => {
                error!("Article not found, dropping message: {}", article_uuid);
                msg.ack().await.ok();
            }

            Err(err) => {
                error!("Processing failed: {}, retrying later in 24 hours", err);

                msg.ack_with(jetstream::AckKind::Nak(Some(Duration::from_secs(24 * 60 * 60))))
                    .await
                    .ok();
            }
        }
    }
}
