use serde::{Deserialize, Serialize};

// probably doesn't need to just be one type here
#[derive(Serialize, Deserialize)]
pub struct Bother {
    pub user_id: String,                 // TODO: use uuid
    pub conversation_id: Option<String>, // TODO: use uuid
    pub message: String,
}
