use axum::{http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::model::bother::Bother;

// TODO: use extenstion for with db
pub async fn bother_blockito(axum::Json(bother): axum::Json<Bother>) -> impl IntoResponse {
    // TODO: expand on system prompt and figure out what data to train with and what model to use
    let messages = vec![json!({
        "role": "system",
        "content": "your name is blockito and you are neat"
    })];
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
