use anyhow::anyhow;
use async_nats::jetstream;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tokio_stream::StreamExt;
use tracing::log::error;
use uuid::Uuid;

use crate::entities::{article, article_data};
use crate::dto::worker_dto::{QueueStats, ConsumerStats};

const TASKS_ML_SENTIMENTAL_ANALYSIS_SUBJECT: &str = "tasks.ml.sentimental-analysis";
const TASKS_ML_CATEGORIZE: &str = "tasks.ml.categorize";

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

pub async fn calculate_sentimental_analysis_for_uuid(js: &jetstream::Context, article_uuid: Uuid) -> Result<(), anyhow::Error> {
    publish_task_for_article(js, TASKS_ML_SENTIMENTAL_ANALYSIS_SUBJECT, article_uuid).await
}

pub async fn categorize_article_for_uuid(js: &jetstream::Context, article_uuid: Uuid) -> Result<(), anyhow::Error> {
    publish_task_for_article(js, TASKS_ML_CATEGORIZE, article_uuid).await
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
    calculate_sentimental_analysis_for_uuid(js, article_uuid).await?;
    categorize_article_for_uuid(js, article_uuid).await?;

    Ok(())
}

pub async fn run_ml(db: &DatabaseConnection, js: &jetstream::Context, missing_only: bool) -> Result<(), anyhow::Error>{
    calculate_sentimental_analysis(db, js, missing_only).await?;
    categorize_articles(db, js, missing_only).await?;

    Ok(())
}

pub async fn categorize_articles(
    db: &DatabaseConnection,
    js: &jetstream::Context,
    missing_only: bool,
) -> Result<(), anyhow::Error> {
    let mut articles_query = article::Entity::find().left_join(article_data::Entity);

    if missing_only {
        articles_query = articles_query.filter(
            sea_orm::Condition::any()
                .add(article_data::Column::CategoryId.is_null())
                .add(article_data::Column::Id.is_null()),
        );
    }

    let articles = articles_query.all(db).await.unwrap();

    for article in articles {
        categorize_article_for_uuid(js, article.id).await?;
    }

    Ok(())
}

pub async fn calculate_sentimental_analysis(
    db: &DatabaseConnection,
    js: &jetstream::Context,
    missing_only: bool,
) -> Result<(), anyhow::Error> {
    let mut articles_query = article::Entity::find().left_join(article_data::Entity);

    if missing_only {
        articles_query = articles_query.filter(
            sea_orm::Condition::any()
                .add(article_data::Column::SentimentScore.is_null())
                .add(article_data::Column::Id.is_null()),
        );
    }

    let articles = articles_query.all(db).await.unwrap();

    for article in articles {
        calculate_sentimental_analysis_for_uuid(js, article.id).await?;
    }

    Ok(())
}
