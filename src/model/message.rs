use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(FromRow, Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    id: String,
    conversation_id: String,
    created_at: String, // TODO: use chrono::DateTime
    message: String,
    sender_type: String,
}
