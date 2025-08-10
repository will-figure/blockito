use axum::{http::StatusCode, response::IntoResponse};
use serde_json::json;
use std::sync::Arc;

use crate::db::database::Database;
use crate::model::bother::Bother;

const USER: &str = "user";
const ASSISTANT: &str = "assistant";
const ROBOT_NAME: &str = "blockito";

// TODO: use extenstion for with db
pub async fn bother_blockito(
    axum::Extension(db): axum::Extension<Arc<Database>>,
    axum::Json(bother): axum::Json<Bother>,
) -> impl IntoResponse {
    // if no conversation_id, create a new conversation
    // store initial message
    // store llama response
    // return as expected
    let conversation_id = if bother.conversation_id.is_none() {
        // TODO: remove unwrap
        db.create_conversation(&bother.user_id, "temp title, we'll figure this out later")
            .await
            .unwrap()
    } else {
        bother.conversation_id.unwrap()
    };

    let conversation = db.get_conversation_by_id(&conversation_id).await.unwrap(); // TODO: remove unwrap

    // TODO: expand on system prompt and figure out what data to train with and what model to use
    let mut messages = vec![json!({
        "role": "system",
        "content": "your name is blockito and you are neat"
    })];

    for message in conversation {
        messages.push(json!({
            "role": message.sender_type,
            "content": message.message
        }));
    }

    // db.add_message_to_conversation(USER);

    // get the conversation messages from the database
    // TODO: chat history should be stored/added
    // TODO push in all historical
    let request = json!({
        "model": "PLACEHOLDER",
        "prompt": bother.message,
        "messages": messages,
    });

    let client = reqwest::Client::new();
    let body = client
        .post("http://127.0.0.1:8012/chat/completions")
        .json(&request)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap(); // TODO: no unwrap
    // TODO: parse the response and return something useful
    (StatusCode::OK, axum::Json(body))
}
