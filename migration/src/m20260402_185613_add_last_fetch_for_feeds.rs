use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Feed::Table)
                    .add_column(
                        // Using the enum variant instead of a string alias
                        ColumnDef::new(Feed::LastFetch)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Feed::Table)
                    .drop_column(Feed::LastFetch)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Feed {
    Table,
    // Add the column identifier here so it can be used in up() and down()
    #[sea_orm(iden = "last_fetch")]
    LastFetch,
}