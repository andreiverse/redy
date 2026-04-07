use sea_orm_migration::{prelude::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add the Name column
        manager
            .alter_table(
                Table::alter()
                    .table(Feed::Table)
                    .add_column(ColumnDef::new(Feed::Name).string())
                    .add_column(
                        ColumnDef::new(Feed::DefaultLanguage)
                            .string() // TEXT column, covers ALL languages
                            .default("en-US"),
                    )
                    .to_owned(),
            )
            .await?;

        // Populate Name from Url
        manager
            .exec_stmt(
                Query::update()
                    .table(Feed::Table)
                    .value(Feed::DefaultLanguage, "en-US")
                    .value(Feed::Name, Expr::col(Feed::Url))
                    .to_owned(),
            )
            .await?;

        // Set default language for existing rows
        manager
            .alter_table(
                Table::alter()
                    .table(Feed::Table)
                    .modify_column(
                        ColumnDef::new(Feed::Name).string().not_null()
                    )
                    .modify_column(
                        ColumnDef::new(Feed::DefaultLanguage).string().not_null()
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Feed::Table)
                    .drop_column(Feed::Name)
                    .drop_column(Feed::DefaultLanguage)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Feed {
    Table,
    Name,
    Url,
    DefaultLanguage,
}