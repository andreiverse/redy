use sea_orm_migration::{prelude::*, sea_orm::DbBackend};
use sea_orm::Statement; // runtime statement

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Add nullable column
        manager
            .alter_table(
                Table::alter()
                    .table(Article::Table)
                    .add_column(ColumnDef::new(Article::Language).string())
                    .to_owned(),
            )
            .await?;

        // 2. Populate language from feed (raw SQL)
        let sql = r#"
            UPDATE article
            SET language = feed.default_language
            FROM feed
            WHERE article.feed_id = feed.id
        "#;
        manager.get_connection().execute(Statement::from_string(
            DbBackend::Postgres,
            sql.to_string(),
        )).await?;

        // 3. Make column NOT NULL
        manager
            .alter_table(
                Table::alter()
                    .table(Article::Table)
                    .modify_column(ColumnDef::new(Article::Language).string().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Article::Table)
                    .drop_column(Article::Language)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum Article {
    Table,
    Language,
    FeedId, // if your join requires feed_id
}

#[derive(DeriveIden)]
pub enum Feed {
    Table,
    Id,
    DefaultLanguage,
}