use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use crate::{
    error::ApiError,
    models::AppState,
    routes::status::ApiResponse,
};

/// Config response matching frontend's Config interface
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub protonvpn_server: String,
    pub tailscale_hostname: String,
    pub auto_connect: bool,
    pub advertise_exit_node: bool,
}

/// Config update request from frontend
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigUpdateRequest {
    pub protonvpn_server: Option<String>,
    pub tailscale_hostname: Option<String>,
    pub auto_connect: Option<bool>,
    pub advertise_exit_node: Option<bool>,
}

pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let config = &state.config;

    // Reflect live exit node state in the config response
    let exit_node_status = state.get_exit_node_status().await;

    // Try to read actual settings from the running container
    let (live_server, live_hostname) = match state.docker.get_container_env().await {
        Ok(env) => {
            let server = env.get("PROTON_WG_ENDPOINT").map(|ep| {
                // Strip port suffix (e.g., "1.2.3.4:51820" -> "1.2.3.4")
                ep.split(':').next().unwrap_or(ep).to_string()
            });
            let hostname = env.get("TAILSCALE_HOSTNAME").cloned();
            (server, hostname)
        }
        Err(e) => {
            warn!("Failed to read container env, using static config: {}", e);
            (None, None)
        }
    };

    let response = ConfigResponse {
        protonvpn_server: live_server.unwrap_or_else(|| config.vpn.default_server.clone()),
        tailscale_hostname: live_hostname
            .unwrap_or_else(|| crate::services::docker::CONTAINER_NAME.to_string()),
        auto_connect: config.vpn.auto_connect,
        advertise_exit_node: exit_node_status.advertised || config.tailscale.advertise_exit_node,
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        error: None,
    }))
}

pub async fn update_config(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<ConfigUpdateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Config update request: server={:?}, hostname={:?}", request.protonvpn_server, request.tailscale_hostname);

    // For now, return the current config (config updates would require restart)
    let config = &_state.config;

    let response = ConfigResponse {
        protonvpn_server: request.protonvpn_server.unwrap_or_else(|| config.vpn.default_server.clone()),
        tailscale_hostname: request.tailscale_hostname.unwrap_or_else(|| config.tailscale.container_name.clone()),
        auto_connect: request.auto_connect.unwrap_or(config.vpn.auto_connect),
        advertise_exit_node: request.advertise_exit_node.unwrap_or(config.tailscale.advertise_exit_node),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        error: None,
    }))
}
