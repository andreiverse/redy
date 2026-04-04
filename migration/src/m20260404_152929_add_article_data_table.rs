use sea_orm_migration::{prelude::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ArticleData::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ArticleData::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ArticleData::SentimentScore).double())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-articledata-article-id")
                            .from(ArticleData::Table, ArticleData::Id)
                            .to(Article::Table, Article::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ArticleData::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ArticleData {
    Table,
    Id,
    SentimentScore,
}

#[derive(DeriveIden)]
enum Article {
    Table,
    Id,
}