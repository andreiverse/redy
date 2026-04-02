use async_nats::jetstream::{self, context::PublishAckFuture};
use chrono::{Duration, Utc};
use reqwest::Url;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ExprTrait, QueryFilter,
    Set, TryIntoModel,
};
use std::{result::Result::Ok, str::FromStr};
use tracing::{error, info};

use crate::{
    entities::{article, feed},
    service::rss_fetcher_service,
};

pub async fn fetch_feeds_task(
    db: &DatabaseConnection,
    jetstream_context: &jetstream::Context,
) -> Result<(), anyhow::Error> {
    info!("Running fetch feeds task");

    // Only get feeds that haven't been fetched in the last minute (or never)
    let one_minute_ago = Utc::now() - Duration::minutes(1);

    let feeds = feed::Entity::find()
        .filter(
            feed::Column::LastFetch
                .is_null()
                .or(feed::Column::LastFetch.lt(one_minute_ago)),
        )
        .all(db)
        .await?;

    for feed in feeds {
        // TODO: difference between last try and last successful fetch
        let mut active_feed: feed::ActiveModel = feed.clone().into();
        active_feed.last_fetch = Set(Some(Utc::now().into()));

        if let Err(e) = active_feed.update(db).await {
            error!("Failed to update last_fetch for feed {}: {:?}", feed.id, e);
            continue;
        }

        if let Err(e) = handle_feed(db, jetstream_context, &feed).await {
            error!("Error processing feed {}: {:?}", feed.url, e);
        }
    }

    Ok(())
}

pub async fn handle_feed(
    db: &DatabaseConnection,
    jetstream_context: &jetstream::Context,
    feed: &feed::Model,
) -> Result<(), anyhow::Error> {
    info!("Fetching RSS for: {}", feed.url);

    let articles = rss_fetcher_service::rss_fetch(feed).await?;

    for new_article in articles {
        // We clone here because SeaORM's ActiveModel consumes the value on insert
        let article_active = new_article.clone();

        // "Insert-and-Queue"
        let result = article::Entity::insert(article_active)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(article::Column::ContentHash)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await;

        match result {
            Err(DbErr::RecordNotInserted) => {
                // Article already exists in DB, so we don't need to queue it again
            }
            Ok(_) => {
                // Convert the ActiveModel back to a Model to access fields safely
                let model = new_article.try_into_model().unwrap();
                let url_str = &model.link;

                if let Ok(url) = Url::parse(url_str) {
                    let host = url.host_str().unwrap_or("unknown").replace('.', "_");
                    let subject = format!("tasks.scrape.{}", host);

                    info!(
                        "New article discovered: {}. Queueing in subject: {}",
                        url_str, subject
                    );

                    // Serialize your payload (e.g., as JSON or just the URL string)
                    let payload = url_str.as_bytes().to_vec();
                    let js_res = jetstream_context.publish(subject, payload.into()).await;
                    if let Err(e) = js_res {
                        error!("NATS publish failed for article {}: {:?}", model.id, e);
                    }
                }
            }
            Err(e) => error!("Failed to save article to DB: {:?}", e),
        }
    }

    Ok(())
}
