use std::time::{Duration, Instant};

use async_nats::jetstream::{self, stream};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel,
};
use tokio_stream::StreamExt;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    entities::article,
    metrics,
    service::{scrape_article_service, worker_service},
};

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

    // Idempotency check: if we already have content, don't scrape again
    if article.html_content.is_some() {
        return Ok(HandleResult::Success);
    }

    let article_link = article.link.clone();
    let mut active_article = article.into_active_model();

    let html_content = scrape_article_service::parse_article_from_url(&article_link).await?;

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
                ack_wait: Duration::from_secs(5 * 60), // Increase to 5 minutes to prevent redelivery during slow scrapes
                ..Default::default()
            },
        )
        .await
        .expect("Should be able to create consumer");

    let mut messages = consumer.messages().await.unwrap();

    info!("Scraper worker started. Waiting for messages...");

    while let Some(Ok(msg)) = messages.next().await {
        let start = Instant::now();
        let article_uuid = match Uuid::from_slice(&msg.payload) {
            Ok(uuid) => uuid,
            Err(e) => {
                error!("Invalid UUID bytes (len={}): {}", msg.payload.len(), e);
                if let Err(e) = msg.ack().await {
                    error!("Failed to ack invalid message: {}", e);
                }
                metrics::record_worker_task("scrape_article", false, start.elapsed());
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
                // Try to publish all ML tasks. We use a single loop to publish them
                // and if any fail, we Nak the original scrape message so it's retried.
                let mut published_all = true;

                for task in worker_service::get_ml_tasks() {
                    if let Err(e) = worker_service::publish_task_for_article(jetstream_context, task.subject(), article_uuid).await {
                        error!("Couldn't publish {} for {}: {}", task.subject(), article_uuid, e);
                        published_all = false;
                        break;
                    }
                }

                if !published_all {
                    error!("Failed to publish ML tasks for {}, naking with 10 minutes delay", article_uuid);
                    msg.ack_with(jetstream::AckKind::Nak(Some(Duration::from_secs(10 * 60))))
                        .await
                        .ok();
                    metrics::record_worker_task("scrape_article", false, start.elapsed());
                } else {
                    msg.ack().await.ok();
                    metrics::record_worker_task("scrape_article", true, start.elapsed());
                }
            }

            Ok(HandleResult::NotFound) => {
                error!("Article not found, dropping message: {}", article_uuid);
                msg.ack().await.ok();
                metrics::record_worker_task("scrape_article", true, start.elapsed());
            }

            Err(err) => {
                error!("Processing failed for article {}: {}, retrying in 1 hour", article_uuid, err);

                msg.ack_with(jetstream::AckKind::Nak(Some(Duration::from_secs(
                    60 * 60,
                ))))
                .await
                .ok();
                metrics::record_worker_task("scrape_article", false, start.elapsed());
            }
        }
    }
}
