use axum::{Extension, Json};
use axum::{
    Router,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use std::collections::HashMap;
use tokio::net::TcpListener;

const PORT: &str = "8123";

async fn fallback(uri: Uri) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, format!("No route for {uri}!!1"))
}

async fn health() -> impl IntoResponse {
    let mut map = HashMap::new();
    map.insert("status", "ok");
    (StatusCode::OK, axum::Json(map))
}

struct Bother {
    user_id: String,         // TODO: use uuid
    conversation_id: String, // TODO: use uuid
    messages: String,
}

// TODO: use extenstion for with db
async fn bother_blockito() -> impl IntoResponse {
    // TODO: chat history should be stored/addedadded
    let messages = vec![json!({
        "role": "system",
        "content": "your name is blockito and you are neat"
    })];
    // TODO push in all historical
    let request = json!({
        "model": "PLACEHOLDER",
        "prompt": "what's your favorite cheese?",
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
    println!("response body: {:?}", body);
    // TODO: parse the response and return something useful
    (StatusCode::OK, body)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!!!!!!");

    let llama_router = Router::new()
        // TODO: shouldn't be a get
        .route("/", get(health))
        .route("/health", get(health))
        .route("/bother", get(bother_blockito))
        .fallback(fallback);

    let listener = TcpListener::bind(format!("127.0.0.1:{PORT}")).await?;

    axum::serve(listener, llama_router).await?;
    Ok(())
}
