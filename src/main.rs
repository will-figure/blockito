mod http;
mod model;

use axum::{
    Router,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
};
use tokio::net::TcpListener;

use crate::http::health::health;

use crate::http::bother::bother_blockito;

const PORT: &str = "8123";

async fn fallback(uri: Uri) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, format!("No route for {uri}!!1"))
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
