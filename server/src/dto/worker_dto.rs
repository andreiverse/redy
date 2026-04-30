use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
