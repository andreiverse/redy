use anyhow::anyhow;
use async_nats::jetstream;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter};
use tokio_stream::StreamExt;
use tracing::info;
use tracing::log::error;
use uuid::Uuid;

use crate::entities::{article, article_data};
use crate::dto::worker_dto::{QueueStats, ConsumerStats, RescheduleRequest, TaskType, WorkerTaskPayload};

pub trait WorkerTask: Send + Sync {
    fn name(&self) -> TaskType;
    fn subject(&self) -> &'static str;
    fn missing_condition(&self) -> sea_orm::Condition;
    fn is_missing(&self, article: &article::Model, article_data: Option<&article_data::Model>) -> bool;
}

pub struct ScrapeTask;
impl WorkerTask for ScrapeTask {
    fn name(&self) -> TaskType {
        TaskType::Scrape
    }
    fn subject(&self) -> &'static str {
        "tasks.scrape.article"
    }
    fn missing_condition(&self) -> sea_orm::Condition {
        sea_orm::Condition::any()
            .add(article::Column::HtmlContent.is_null())
    }
    fn is_missing(&self, article: &article::Model, _article_data: Option<&article_data::Model>) -> bool {
        article.html_content.is_none()
    }
}

pub struct SentimentalAnalysisTask;
impl WorkerTask for SentimentalAnalysisTask {
    fn name(&self) -> TaskType {
        TaskType::SentimentalAnalysis
    }
    fn subject(&self) -> &'static str {
        "tasks.ml.sentimental-analysis"
    }
    fn missing_condition(&self) -> sea_orm::Condition {
        sea_orm::Condition::any()
            .add(article_data::Column::SentimentScore.is_null())
            .add(article_data::Column::Id.is_null())
    }
    fn is_missing(&self, _article: &article::Model, article_data: Option<&article_data::Model>) -> bool {
        article_data.map(|d| d.sentiment_score.is_none()).unwrap_or(true)
    }
}

pub struct CategorizeTask;
impl WorkerTask for CategorizeTask {
    fn name(&self) -> TaskType {
        TaskType::Categorize
    }
    fn subject(&self) -> &'static str {
        "tasks.ml.categorize"
    }
    fn missing_condition(&self) -> sea_orm::Condition {
        sea_orm::Condition::any()
            .add(article_data::Column::CategoryId.is_null())
            .add(article_data::Column::Id.is_null())
    }
    fn is_missing(&self, _article: &article::Model, article_data: Option<&article_data::Model>) -> bool {
        article_data.map(|d| d.category_id.is_none()).unwrap_or(true)
    }
}

pub fn get_ml_tasks() -> Vec<Box<dyn WorkerTask>> {
    vec![
        Box::new(SentimentalAnalysisTask),
        Box::new(CategorizeTask),
    ]
}

pub fn get_all_tasks() -> Vec<Box<dyn WorkerTask>> {
    vec![
        Box::new(ScrapeTask),
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
    extra: serde_json::Map<String, serde_json::Value>,
) -> Result<(), anyhow::Error> {
    let mut headers = async_nats::HeaderMap::new();
    // Unique ID for deduplication: task_subject:article_uuid
    headers.insert("Nats-Msg-Id", format!("{}:{}", subject, uuid).as_str());

    let payload = WorkerTaskPayload {
        article_uuid: Some(uuid),
        extra,
    };

    let payload_bytes = serde_json::to_vec(&payload)?;

    if let Err(e) = js
        .publish_with_headers(subject.to_owned(), headers, payload_bytes.into())
        .await
    {
        error!("Couldn't publish {} to {}: {}", uuid, subject, e);
        return Err(anyhow!(e));
    }

    Ok(())
}

pub async fn run_ml_for_uuid(js: &jetstream::Context, article_uuid: Uuid) -> Result<(), anyhow::Error> {
    for task in get_ml_tasks() {
        publish_task_for_article(js, task.subject(), article_uuid, serde_json::Map::new()).await?;
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
        publish_task_for_article(js, task.subject(), article.id, serde_json::Map::new()).await?;
    }

    Ok(())
}

pub async fn reschedule_articles(
    db: &DatabaseConnection,
    js: &jetstream::Context,
    req: RescheduleRequest,
) -> Result<(), anyhow::Error> {
    info!("Rescheduling articles with request: {:?}", req);

    let mut articles_query = article::Entity::find()
        .left_join(article_data::Entity)
        .select_also(article_data::Entity);

    if let Some(uuid) = req.article_uuid {
        articles_query = articles_query.filter(article::Column::Id.eq(uuid));
    }

    if let Some(feed_uuid) = req.feed_uuid {
        articles_query = articles_query.filter(article::Column::FeedId.eq(feed_uuid));
    }

    if let Some(from) = req.from_date {
        articles_query = articles_query.filter(article::Column::FetchedAt.gte(from));
    }

    if let Some(to) = req.to_date {
        articles_query = articles_query.filter(article::Column::FetchedAt.lte(to));
    }

    let all_available_tasks = get_all_tasks();
    let tasks_to_run: Vec<&dyn WorkerTask> = if let Some(req_tasks) = &req.tasks {
        all_available_tasks
            .iter()
            .filter(|t| req_tasks.contains(&t.name()))
            .map(|t| t.as_ref())
            .collect()
    } else {
        all_available_tasks.iter().map(|t| t.as_ref()).collect()
    };

    let missing_only = req.missing_only.unwrap_or(false);

    if missing_only {
        let mut cond = Condition::any();
        for task in &tasks_to_run {
            cond = cond.add(task.missing_condition());
        }
        articles_query = articles_query.filter(cond);
    }

    let articles = articles_query.all(db).await?;

    for (article_model, article_data_opt) in articles {
        for task in &tasks_to_run {
            if task.name() == TaskType::Scrape && !missing_only {
                let mut active_article = article_model.clone().into_active_model();
                active_article.html_content = Set(None);
                active_article.html_content_from_feed = Set(false);
                active_article.update(db).await?;

                publish_task_for_article(js, task.subject(), article_model.id, serde_json::Map::new()).await?;
            } else if !missing_only || task.is_missing(&article_model, article_data_opt.as_ref()) {
                publish_task_for_article(js, task.subject(), article_model.id, serde_json::Map::new()).await?;
            }
        }
    }

    Ok(())
}

