use anyhow::anyhow;
use async_nats::jetstream;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tokio_stream::StreamExt;
use tracing::info;
use tracing::log::error;
use uuid::Uuid;

use crate::entities::{article, article_data};
use crate::dto::worker_dto::{QueueStats, ConsumerStats};

pub trait WorkerTask: Send + Sync {
    fn name(&self) -> &'static str;
    fn subject(&self) -> &'static str;
    fn missing_condition(&self) -> sea_orm::Condition;
}

pub struct SentimentalAnalysisTask;
impl WorkerTask for SentimentalAnalysisTask {
    fn name(&self) -> &'static str {
        "SentimentalAnalysis"
    }
    fn subject(&self) -> &'static str {
        "tasks.ml.sentimental-analysis"
    }
    fn missing_condition(&self) -> sea_orm::Condition {
        sea_orm::Condition::any()
            .add(article_data::Column::SentimentScore.is_null())
            .add(article_data::Column::Id.is_null())
    }
}

pub struct CategorizeTask;
impl WorkerTask for CategorizeTask {
    fn name(&self) -> &'static str {
        "Categorize"
    }
    fn subject(&self) -> &'static str {
        "tasks.ml.categorize"
    }
    fn missing_condition(&self) -> sea_orm::Condition {
        sea_orm::Condition::any()
            .add(article_data::Column::CategoryId.is_null())
            .add(article_data::Column::Id.is_null())
    }
}

pub fn get_ml_tasks() -> Vec<Box<dyn WorkerTask>> {
    vec![
        Box::new(SentimentalAnalysisTask),
        Box::new(CategorizeTask),
    ]
}

pub async fn get_all_queue_stats(js: &jetstream::Context) -> Result<Vec<QueueStats>, anyhow::Error> {
    let mut all_stats = Vec::new();
    let mut names = js.stream_names();

    while let Some(Ok(stream_name)) = names.next().await {
        if let Ok(stats) = get_stream_stats(js, &stream_name).await {
            all_stats.push(stats);
        }
    }

    Ok(all_stats)
}

pub async fn get_stream_stats(js: &jetstream::Context, stream_name: &str) -> Result<QueueStats, anyhow::Error> {
    let mut stream = js.get_stream(stream_name).await?;
    let info = stream.info().await?;
    let messages = info.state.messages;
    let bytes = info.state.bytes;
    let consumer_count = info.state.consumer_count;

    let mut consumers = Vec::new();
    let mut consumer_names = stream.consumer_names();

    while let Some(Ok(consumer_name)) = consumer_names.next().await {
        if let Ok(c_info) = stream.consumer_info(&consumer_name).await {
            consumers.push(ConsumerStats {
                name: consumer_name,
                pending: c_info.num_pending,
                ack_pending: c_info.num_ack_pending,
                redelivered: c_info.num_redelivered,
            });
        }
    }

    Ok(QueueStats {
        stream_name: stream_name.to_string(),
        messages,
        bytes,
        consumer_count,
        consumers,
    })
}

pub async fn publish_task_for_article(
    js: &jetstream::Context,
    subject: &str,
    uuid: Uuid,
) -> Result<(), anyhow::Error> {
    if let Err(e) = js
        .publish(subject.to_owned(), uuid.as_bytes().to_vec().into())
        .await
    {
        error!("Couldn't publish {} to {}: {}", uuid, subject, e);
        return Err(anyhow!(e));
    }

    Ok(())
}

pub async fn run_ml_for_uuid(js: &jetstream::Context, article_uuid: Uuid) -> Result<(), anyhow::Error> {
    for task in get_ml_tasks() {
        publish_task_for_article(js, task.subject(), article_uuid).await?;
    }

    Ok(())
}

pub async fn run_ml(db: &DatabaseConnection, js: &jetstream::Context, missing_only: bool) -> Result<(), anyhow::Error>{
    for task in get_ml_tasks() {
        run_task_for_articles(db, js, task.as_ref(), missing_only).await?;
    }

    Ok(())
}

pub async fn run_task_for_articles(
    db: &DatabaseConnection,
    js: &jetstream::Context,
    task: &dyn WorkerTask,
    missing_only: bool,
) -> Result<(), anyhow::Error> {
    info!("Scheduling task: {} (missing_only: {})", task.name(), missing_only);
    let mut articles_query = article::Entity::find().left_join(article_data::Entity);

    if missing_only {
        articles_query = articles_query.filter(task.missing_condition());
    }

    let articles = articles_query.all(db).await.unwrap();

    for article in articles {
        publish_task_for_article(js, task.subject(), article.id).await?;
    }

    Ok(())
}

