use async_nats::jetstream;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::log::error;

use crate::entities::{article, article_data};

#[allow(dead_code)]
pub async fn calculate_sentimental_analysis_on_missing(
    db: &DatabaseConnection,
    js: &jetstream::Context,
) {
    let articles = article::Entity::find()
        .left_join(article_data::Entity)
        .filter(
            sea_orm::Condition::any()
                .add(article_data::Column::SentimentScore.is_null())
                .add(article_data::Column::Id.is_null()),
        )
        .all(db)
        .await
        .unwrap();

    for article in articles {
        if let Err(e) = js
            .publish(
                "tasks.ml.sentimental-analysis",
                article.id.as_bytes().to_vec().into(),
            )
            .await
        {
            error!("Couldn't publish sentimental analyis {}", e);
        }
    }
}
