use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::models::AppState;

pub async fn health_check(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    StatusCode::OK
}
