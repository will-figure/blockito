use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(FromRow, Deserialize, Serialize, Debug, Clone)]
pub struct Conversation {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub created_at: String, // TODO: use chrono::DateTime
}
