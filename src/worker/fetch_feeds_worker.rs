use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ExprTrait, QueryFilter, Set
};
use std::result::Result::Ok;
use tracing::{error, info};

use crate::{
    entities::{article, feed},
    service::rss_fetcher_service,
};

pub async fn fetch_feeds_task(db: &DatabaseConnection) -> Result<(), anyhow::Error> {
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

        // We spawn or await here. Awaiting is safer for strict rate limiting.
        if let Err(e) = handle_feed(db, &feed).await {
            error!("Error processing feed {}: {:?}", feed.url, e);
        }
    }

    Ok(())
}

pub async fn handle_feed(db: &DatabaseConnection, feed: &feed::Model) -> Result<(), anyhow::Error> {
    info!("Fetching RSS for: {}", feed.url);

    let articles = rss_fetcher_service::rss_fetch(feed).await?;

    for new_article in articles {
        // "Insert-and-Queue"
        let result = article::Entity::insert(new_article)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(article::Column::ContentHash)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await;

        match result {
            Err(DbErr::RecordNotInserted) => {
                // This is fine! It just means the article already exists.
                // We can ignore this "error".
            }
            Ok(_) => info!("New article discovered and saved."),
            Err(e) => error!("Failed to save article: {:?}", e),
        }
    }

    Ok(())
}
