use async_nats::jetstream;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::log::error;
use uuid::Uuid;

use crate::entities::{article, article_data};

#[allow(dead_code)]
pub async fn recalculate_sentimental_analysis_for_uuid(
    db: &DatabaseConnection,
    js: &jetstream::Context,
    article_uuid: Uuid,
) {
    let article = article::Entity::find_by_id(article_uuid).one(db).await.unwrap().unwrap();

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

#[allow(dead_code)]
pub async fn recalculate_sentimental_analysis(
    db: &DatabaseConnection,
    js: &jetstream::Context,
    missing_only: bool,
) {
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
