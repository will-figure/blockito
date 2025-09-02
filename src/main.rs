mod consts;
mod db;
mod http;
mod model;
mod vector;

use axum::{
    Extension, Router,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
};
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::consts::{EMBEDDING_MODEL, LANGUAGE_MODEL, ROBOT_NAME};
use crate::db::database::Database;
use crate::http::{bother::bother_blockito, conversations::conversations, health::health};
use crate::vector::embedding::Embedding;

const PORT: &str = "8123";

async fn fallback(uri: Uri) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, format!("No route for {uri}!!1"))
}

// TODO: better logging
// TODO: maybe create an encapsulated llama client that handles the language model requests
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = Arc::new(Database::new().await?);
    println!("Database initialized");
    let embedding = Arc::new(Embedding::new().await?);
    println!("embedding initialized");

    // this whole thing is basically preloading a vector database
    // we'll eventually add some checks to see if we need to do it or not

    // get conversations (per user)
    // get conversation state (per conversation id)
    let llama_router = Router::new()
        .route("/", get(health))
        .route("/health", get(health))
        .route("/bother", post(bother_blockito))
        .route("/conversations/{user_id}", get(conversations))
        // GET /conversation/{conversation_id}
        .fallback(fallback)
        .layer(Extension(state))
        .layer(Extension(embedding)); // TODO: remove this and just use the embedding db

    let listener = TcpListener::bind(format!("127.0.0.1:{PORT}")).await?;
    println!("Using embedding model: {EMBEDDING_MODEL}");
    println!("Using language model: {LANGUAGE_MODEL}");
    println!("Robot name: {ROBOT_NAME}!");
    println!("Port: {PORT}");

    axum::serve(listener, llama_router).await?;
    Ok(())
}
