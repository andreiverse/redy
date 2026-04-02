use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchArticleHtmlJob {
    pub article_id: Uuid,
}

pub async fn handle_fetch_article_html_job(
    job: FetchArticleHtmlJob,
) -> Result<(), anyhow::Error> {
    info!("executing job"); 
    // svc.fetch_html_content(job.article_id).await
    //     .map_err(|e| Error::Failed(Box::new(e)))?;
    Ok(())
}
