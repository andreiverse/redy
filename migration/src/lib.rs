pub use sea_orm_migration::prelude::*;


pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260310_212653_create_rss_feed_table::Migration),
            Box::new(m20260401_184207_add_articles::Migration),
        ]
    }
}
mod m20260310_212653_create_rss_feed_table;
mod m20260401_184207_add_articles;
