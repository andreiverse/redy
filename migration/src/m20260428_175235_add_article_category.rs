use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add category to article_data (for ML-predicted category)
        manager
            .alter_table(
                Table::alter()
                    .table(ArticleData::Table)
                    .add_column(ColumnDef::new(ArticleData::Category).string())
                    .to_owned(),
            )
            .await?;

        // Add category to feed (for source-defined category)
        manager
            .alter_table(
                Table::alter()
                    .table(Feed::Table)
                    .add_column(ColumnDef::new(Feed::Category).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ArticleData::Table)
                    .drop_column(ArticleData::Category)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Feed::Table)
                    .drop_column(Feed::Category)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ArticleData {
    Table,
    Category,
}

#[derive(DeriveIden)]
enum Feed {
    Table,
    Category,
}
