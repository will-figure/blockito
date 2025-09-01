use axum::{http::StatusCode, response::IntoResponse};

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let error_message = format!("Internal Server Error: {}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, error_message).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
