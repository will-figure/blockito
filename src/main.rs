mod db;
mod http;
mod model;

use axum::{
    Extension, Router,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
};
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::db::database::Database;
use crate::http::{bother::bother_blockito, health::health};

const PORT: &str = "8123";

async fn fallback(uri: Uri) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, format!("No route for {uri}!!1"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!!!!!!");
    let state = Arc::new(Database::new().await?);

    // TODO: better abstraction

    let llama_router = Router::new()
        .route("/", get(health))
        .route("/health", get(health))
        .route("/bother", post(bother_blockito))
        .fallback(fallback)
        .layer(Extension(state));

    let listener = TcpListener::bind(format!("127.0.0.1:{PORT}")).await?;

    axum::serve(listener, llama_router).await?;
    Ok(())
}
