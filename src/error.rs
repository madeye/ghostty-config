use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("CLI error: {0}")]
    Cli(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Cli(msg) => (StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::Config(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Internal(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        tracing::error!(%status, %message, "request error");
        (status, message).into_response()
    }
}
