use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    Json as AxumJson,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use crate::{
    error::ApiError,
    models::AppState,
};

#[derive(Deserialize, Debug)]
pub struct ConnectRequest {
    pub server: Option<String>,
    pub protocol: Option<String>,
}

#[derive(Serialize)]
pub struct ConnectResponse {
    pub success: bool,
    pub message: String,
}

pub async fn connect(
    State(state): State<Arc<AppState>>,
    AxumJson(request): AxumJson<ConnectRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Connect request received: {:?}", request);
    
    match state.docker.start_vpn(&request).await {
        Ok(_) => {
            info!("VPN containers started successfully");
            let response = ConnectResponse {
                success: true,
                message: "VPN connection initiated".to_string(),
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            warn!("Failed to start VPN: {}", e);
            Err(ApiError::Docker(format!("Failed to start VPN: {}", e)))
        }
    }
}

#[derive(Serialize)]
pub struct DisconnectResponse {
    pub success: bool,
    pub message: String,
}

pub async fn disconnect(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Disconnect request received");
    
    match state.docker.stop_vpn().await {
        Ok(_) => {
            info!("VPN containers stopped successfully");
            let response = DisconnectResponse {
                success: true,
                message: "VPN disconnected successfully".to_string(),
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            warn!("Failed to stop VPN: {}", e);
            Err(ApiError::Docker(format!("Failed to stop VPN: {}", e)))
        }
    }
}
