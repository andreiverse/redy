use sea_orm_migration::{prelude::*, schema::*};
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // create enum types first
        manager
            .create_type(
                Type::create()
                    .as_enum(FeedType::Type)
                    .values([FeedType::Rss])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(ArticleStatus::Type)
                    .values([
                        ArticleStatus::Pending,
                        ArticleStatus::Extracted,
                        ArticleStatus::ExtractionFailed,
                        ArticleStatus::Done,
                    ])
                    .to_owned(),
            )
            .await?;

        // feeds table
        manager
            .create_table(
                Table::create()
                    .table(Feed::Table)
                    .if_not_exists()
                    .col(pk_uuid(Feed::Id))
                    .col(string_uniq(Feed::Url))
                    .col(
                        ColumnDef::new(Feed::FeedType)
                            .custom(Alias::new("feed_type"))
                            .not_null(),
                    )
                    .col(timestamp_with_time_zone(Feed::CreatedAt))
                    .to_owned(),
            )
            .await?;

        // articles table
        manager
            .create_table(
                Table::create()
                    .table(Article::Table)
                    .if_not_exists()
                    .col(pk_uuid(Article::Id))
                    .col(uuid(Article::FeedId))
                    .col(string(Article::Title))
                    .col(string_null(Article::FeedDescription))
                    .col(string_uniq(Article::ContentHash))
                    .col(string(Article::Link))
                    .col(text_null(Article::HtmlContent))
                    .col(
                        ColumnDef::new(Article::Status)
                            .custom(Alias::new("article_status"))
                            .not_null(),
                    )
                    .col(timestamp_with_time_zone_null(Article::PublishedAt))
                    .col(timestamp_with_time_zone(Article::FetchedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_article_feed")
                    .from(Article::Table, Article::FeedId)
                    .to(Feed::Table, Feed::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // article_data table
        manager
            .create_table(
                Table::create()
                    .table(ArticleData::Table)
                    .if_not_exists()
                    .col(pk_uuid(ArticleData::Id))
                    .col(uuid(ArticleData::ArticleId))
                    .col(text_null(ArticleData::Summary))
                    .col(string_null(ArticleData::Sentiment))
                    .col(string_null(ArticleData::Language))
                    .col(array_null(ArticleData::Keywords, ColumnType::Text))
                    .col(timestamp_with_time_zone_null(ArticleData::ProcessedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_article_data_article")
                    .from(ArticleData::Table, ArticleData::ArticleId)
                    .to(Article::Table, Article::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_article_feed_id")
                    .table(Article::Table)
                    .col(Article::FeedId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_article_status")
                    .table(Article::Table)
                    .col(Article::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_article_published_at")
                    .table(Article::Table)
                    .col(Article::PublishedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_article_data_article_id")
                    .table(ArticleData::Table)
                    .col(ArticleData::ArticleId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ArticleData::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Article::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Feed::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(ArticleStatus::Type).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(FeedType::Type).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Feed {
    Table,
    Id,
    Url,
    FeedType,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Article {
    Table,
    Id,
    FeedId,
    Title,
    FeedDescription,
    ContentHash,
    Link,
    HtmlContent,
    Status,
    PublishedAt,
    FetchedAt,
}

#[derive(DeriveIden)]
enum ArticleData {
    Table,
    Id,
    ArticleId,
    Summary,
    Sentiment,
    Language,
    Keywords,
    ProcessedAt,
}

// enum type identifiers
#[derive(DeriveIden)]
enum FeedType {
    #[sea_orm(iden = "feed_type")]
    Type,
    Rss,
    Atom,
    JsonFeed,
}

#[derive(DeriveIden)]
enum ArticleStatus {
    #[sea_orm(iden = "article_status")]
    Type,
    Pending,
    Extracted,
    ExtractionFailed,
    Done,
}