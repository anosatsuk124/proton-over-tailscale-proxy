use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Docker error: {0}")]
    Docker(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Internal server error")]
    Internal,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_response) = match &self {
            ApiError::Docker(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorResponse {
                    error: "DockerError".to_string(),
                    message: msg.clone(),
                },
            ),
            ApiError::Config(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: "ConfigError".to_string(),
                    message: msg.clone(),
                },
            ),
            ApiError::Io(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: "IoError".to_string(),
                    message: self.to_string(),
                },
            ),
            ApiError::Serialization(_) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: "SerializationError".to_string(),
                    message: self.to_string(),
                },
            ),
            ApiError::Network(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorResponse {
                    error: "NetworkError".to_string(),
                    message: msg.clone(),
                },
            ),
            ApiError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: "InternalError".to_string(),
                    message: "An internal error occurred".to_string(),
                },
            ),
        };

        (status, Json(error_response)).into_response()
    }
}
