use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserFeedFavorite::Table)
                    .if_not_exists()
                    .col(uuid(UserFeedFavorite::UserUuid).not_null())
                    .col(uuid(UserFeedFavorite::FeedUuid).not_null())
                    .primary_key(
                        IndexCreateStatement::new()
                            .name("pk_user_feed")
                            .primary()
                            .col(UserFeedFavorite::UserUuid)
                            .col(UserFeedFavorite::FeedUuid),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk_user_favorite_feed_uuid")
                            .from_tbl(UserFeedFavorite::Table)
                            .from_col(UserFeedFavorite::FeedUuid)
                            .to_tbl(Feed::Table)
                            .to_col(Feed::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk_user_favorite_user_uuid")
                            .from_tbl(UserFeedFavorite::Table)
                            .from_col(UserFeedFavorite::UserUuid)
                            .to_tbl(User::Table)
                            .to_col(User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_feed_favorite_user")
                    .table(UserFeedFavorite::Table)
                    .col(UserFeedFavorite::UserUuid)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_feed_favorite_feed")
                    .table(UserFeedFavorite::Table)
                    .col(UserFeedFavorite::FeedUuid)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserFeedFavorite::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Feed {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum UserFeedFavorite {
    Table,
    UserUuid,
    FeedUuid,
}
