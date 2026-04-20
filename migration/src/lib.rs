pub use sea_orm_migration::prelude::*;


pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260310_212653_create_rss_feed_table::Migration),
            Box::new(m20260401_184207_add_articles::Migration),
            Box::new(m20260402_185613_add_last_fetch_for_feeds::Migration),
            Box::new(m20260403_164713_remove_rss_feed_table::Migration),
            Box::new(m20260404_152805_remove_article_data_table::Migration),
            Box::new(m20260404_152929_add_article_data_table::Migration),
            Box::new(m20260407_190659_add_feed_name::Migration),
            Box::new(m20260407_193740_add_article_language::Migration),
            Box::new(m20260415_071631_create_user_table::Migration),
            Box::new(m20260420_191609_add_feed_ownera::Migration),
        ]
    }
}
mod m20260310_212653_create_rss_feed_table;
mod m20260401_184207_add_articles;
mod m20260402_185613_add_last_fetch_for_feeds;
mod m20260403_164713_remove_rss_feed_table;
mod m20260404_152805_remove_article_data_table;
mod m20260404_152929_add_article_data_table;
mod m20260407_190659_add_feed_name;
mod m20260407_193740_add_article_language;
mod m20260415_071631_create_user_table;
mod m20260420_191609_add_feed_ownera;
