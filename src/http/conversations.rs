use axum::{extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::json;
use std::sync::Arc;

use uuid::Uuid;

use crate::db::database::Database;
use crate::http::error::AppError;

pub async fn conversations(
    axum::Extension(db): axum::Extension<Arc<Database>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // to string seems wrong here...
    let user_id = user_id.to_string();
    let things = db.get_conversation_list_by_user(user_id.as_str()).await?;
    println!("Get conversations for user_id: {}", user_id);
    println!("conversations: {:?}", things);
    let conversations: Vec<Uuid> = vec![];
    Ok((
        StatusCode::OK,
        axum::Json(json!({
            "conversations": conversations
        })),
    ))
}
