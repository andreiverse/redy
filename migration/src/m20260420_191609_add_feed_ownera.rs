use sea_orm_migration::{prelude::*, schema::*};

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
                        ColumnDef::new(Feed::OwnerUuid)
                            .uuid()
                            .null(), // or .not_null() if required
                    )
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_feed_owner_uuid")
                            .from_tbl(Feed::Table)
                            .from_col(Feed::OwnerUuid)
                            .to_tbl(User::Table)
                            .to_col(User::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
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
                    .drop_foreign_key(Alias::new("fk_feed_owner_uuid"))
                    .drop_column(Feed::OwnerUuid)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Feed {
    Table,
    OwnerUuid,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}