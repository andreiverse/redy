use sea_orm_migration::{prelude::*, schema::*, sea_orm::Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create Category table
        manager
            .create_table(
                Table::create()
                    .table(Category::Table)
                    .if_not_exists()
                    .col(
                        pk_uuid(Category::Id)
                            .default(Expr::cust("gen_random_uuid()"))
                            .not_null(),
                    )
                    .col(string(Category::HumanName).not_null())
                    .col(string(Category::ModelDescription).not_null())
                    .col(string(Category::HumanDescription).not_null())
                    .to_owned(),
            )
            .await?;

        // 2. Create FeedCategory junction table
        manager
            .create_table(
                Table::create()
                    .table(FeedCategory::Table)
                    .if_not_exists()
                    .col(uuid(FeedCategory::FeedId).not_null())
                    .col(uuid(FeedCategory::CategoryId).not_null())
                    .col(string_null(FeedCategory::ModelDescriptionOverride))
                    .primary_key(
                        Index::create()
                            .col(FeedCategory::FeedId)
                            .col(FeedCategory::CategoryId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-feed_category-feed_id")
                            .from(FeedCategory::Table, FeedCategory::FeedId)
                            .to(Feed::Table, Feed::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-feed_category-category_id")
                            .from(FeedCategory::Table, FeedCategory::CategoryId)
                            .to(Category::Table, Category::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();

        // 3. Prefill categories from article_data
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "INSERT INTO category (human_name, model_description, human_description) SELECT DISTINCT category, category, category FROM article_data WHERE category IS NOT NULL".to_owned()
        )).await?;

        // 4. Assign categories to feeds if they contain at least one article from that category
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "INSERT INTO feed_category (feed_id, category_id) \
             SELECT DISTINCT a.feed_id, c.id \
             FROM article a \
             JOIN article_data ad ON a.id = ad.id \
             JOIN category c ON ad.category = c.human_name \
             WHERE ad.category IS NOT NULL".to_owned()
        )).await?;

        // 5. Refactor ArticleData: Replace category string with category_id
        // 5.1 Add category_id column
        manager
            .alter_table(
                Table::alter()
                    .table(ArticleData::Table)
                    .add_column(ColumnDef::new(ArticleData::CategoryId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // 5.2 Map existing string categories to IDs
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "UPDATE article_data ad SET category_id = c.id \
             FROM category c \
             WHERE ad.category = c.human_name".to_owned()
        )).await?;

        // 5.3 Add foreign key constraint
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-article_data-category_id")
                    .from(ArticleData::Table, ArticleData::CategoryId)
                    .to(Category::Table, Category::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        // 5.4 Remove old category column
        manager
            .alter_table(
                Table::alter()
                    .table(ArticleData::Table)
                    .drop_column(Alias::new("category"))
                    .to_owned(),
            )
            .await?;
            
        // 6. Refactor Feed: Remove old category column (it's now in FeedCategory)
        manager
            .alter_table(
                Table::alter()
                    .table(Feed::Table)
                    .drop_column(Alias::new("category"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Restore category column to Feed
        manager
            .alter_table(
                Table::alter()
                    .table(Feed::Table)
                    .add_column(string_null(Alias::new("category")))
                    .to_owned(),
            )
            .await?;

        // Restore category column to ArticleData
        manager
            .alter_table(
                Table::alter()
                    .table(ArticleData::Table)
                    .add_column(string_null(Alias::new("category")))
                    .to_owned(),
            )
            .await?;

        // Map IDs back to strings before dropping
        let db = manager.get_connection();
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "UPDATE article_data ad SET category = c.human_name \
             FROM category c \
             WHERE ad.category_id = c.id".to_owned()
        )).await?;

        // Drop foreign key and column
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk-article_data-category_id")
                    .table(ArticleData::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ArticleData::Table)
                    .drop_column(ArticleData::CategoryId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(FeedCategory::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Category::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
#[allow(dead_code)]
enum Feed {
    Table,
    Id,
}

#[derive(DeriveIden)]
#[allow(dead_code)]
enum Article {
    Table,
    Id,
    FeedId,
}

#[derive(DeriveIden)]
#[allow(dead_code)]
enum ArticleData {
    Table,
    Id,
    CategoryId,
}

#[derive(DeriveIden)]
enum Category {
    Table,
    Id,
    HumanName,
    ModelDescription,
    HumanDescription
}


#[derive(DeriveIden)]
enum FeedCategory {
    Table,
    FeedId,
    CategoryId,
    ModelDescriptionOverride
}
