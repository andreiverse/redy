use apalis::prelude::Data;
use apalis_cron::Tick;
use chrono::Local;
use sea_orm::sqlx;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;


#[derive(Debug, Serialize, Deserialize)]
pub struct FetchArticlesJob {

}



pub async fn handle_fetch_articles_job(
    tick: Tick<Local>, data: Data<sqlx::PgPool>
) -> Result<(), anyhow::Error> {    
    let pool = data.acquire().await?;


    
    Ok(())
}