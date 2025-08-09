mod http;
mod model;

use axum::{
    Router,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use std::collections::HashMap;
use tokio::net::TcpListener;

use crate::model::bother::Bother;

const PORT: &str = "8123";

async fn fallback(uri: Uri) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, format!("No route for {uri}!!1"))
}

async fn health() -> impl IntoResponse {
    let mut map = HashMap::new();
    map.insert("status", "ok");
    (StatusCode::OK, axum::Json(map))
}

// TODO: use extenstion for with db
async fn bother_blockito(axum::Json(bother): axum::Json<Bother>) -> impl IntoResponse {
    // TODO: expand on system prompt and figure out what data to train with and what model to use
    let messages = vec![json!({
        "role": "system",
        "content": "your name is blockito and you are neat"
    })];
    // get the conversation messages from the database
    // TODO: chat history should be stored/addedadded
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!!!!!!");

    let llama_router = Router::new()
        .route("/", get(health))
        .route("/health", get(health))
        .route("/bother", post(bother_blockito))
        .fallback(fallback);

    let listener = TcpListener::bind(format!("127.0.0.1:{PORT}")).await?;

    axum::serve(listener, llama_router).await?;
    Ok(())
}
