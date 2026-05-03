use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ScheduleResult {
    pub error: Option<String>,
}

impl Into<ScheduleResult> for Result<(), anyhow::Error> {
    fn into(self) -> ScheduleResult {
        ScheduleResult {
            error: self.err().map(|f| f.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq)]
pub enum TaskType {
    Scrape,
    SentimentalAnalysis,
    Categorize,
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct RescheduleRequest {
    pub article_uuid: Option<Uuid>,
    pub tasks: Option<Vec<TaskType>>,
    pub missing_only: Option<bool>,
    pub from_date: Option<chrono::DateTime<chrono::Utc>>,
    pub to_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct WorkerTaskPayload {
    pub article_uuid: Option<Uuid>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct QueueStats {
    pub stream_name: String,
    pub messages: u64,
    pub bytes: u64,
    pub consumer_count: usize,
    pub consumers: Vec<ConsumerStats>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ConsumerStats {
    pub name: String,
    pub pending: u64,
    pub ack_pending: usize,
    pub redelivered: usize,
}
