use axum::{http::StatusCode, response::IntoResponse};
use std::collections::HashMap;

pub async fn health() -> impl IntoResponse {
    let mut map = HashMap::new();
    map.insert("status", "ok");
    (StatusCode::OK, axum::Json(map))
}
