use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    Json as AxumJson,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};
use crate::{
    error::ApiError,
    models::AppState,
};

/// Request to connect VPN with optional exit node configuration
#[derive(Deserialize, Debug)]
pub struct ConnectRequest {
    /// VPN server to connect to
    pub server: Option<String>,
    /// Protocol to use (udp/tcp)
    pub protocol: Option<String>,
    /// Whether to enable exit node advertisement
    pub enable_exit_node: Option<bool>,
}

/// Response for connect operation
#[derive(Serialize)]
pub struct ConnectResponse {
    /// Whether operation was successful
    pub success: bool,
    /// Response message
    pub message: String,
    /// Exit node status after connection
    pub exit_node_enabled: bool,
}

/// Connect VPN and optionally enable exit node
pub async fn connect(
    State(state): State<Arc<AppState>>,
    AxumJson(request): AxumJson<ConnectRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Connect request received: {:?}", request);

    let enable_exit_node = request.enable_exit_node.unwrap_or(false);

    // Update exit node enabled state
    state
        .update_exit_node_state(|s| {
            s.enabled = enable_exit_node;
        })
        .await;

    // Start VPN containers with exit node configuration
    match state
        .docker
        .start_vpn_with_exit_node(&request, enable_exit_node)
        .await
    {
        Ok(_) => {
            info!("VPN containers started successfully");

            // Check if exit node was successfully enabled
            let exit_node_enabled = if enable_exit_node {
                match state.docker.check_exit_node_advertised().await {
                    Ok(advertised) => advertised,
                    Err(e) => {
                        warn!("Failed to verify exit node status: {}", e);
                        false
                    }
                }
            } else {
                false
            };

            let response = ConnectResponse {
                success: true,
                message: "VPN connection initiated".to_string(),
                exit_node_enabled,
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to start VPN: {}", e);
            Err(ApiError::Docker(format!("Failed to start VPN: {}", e)))
        }
    }
}

/// Response for disconnect operation
#[derive(Serialize)]
pub struct DisconnectResponse {
    /// Whether operation was successful
    pub success: bool,
    /// Response message
    pub message: String,
}

/// Disconnect VPN and cleanup exit node
pub async fn disconnect(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Disconnect request received");

    // Disable exit node first
    state
        .update_exit_node_state(|s| {
            s.enabled = false;
            s.advertised = false;
        })
        .await;

    match state.docker.stop_vpn_with_exit_node().await {
        Ok(_) => {
            info!("VPN containers stopped successfully");
            let response = DisconnectResponse {
                success: true,
                message: "VPN disconnected successfully".to_string(),
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to stop VPN: {}", e);
            Err(ApiError::Docker(format!("Failed to stop VPN: {}", e)))
        }
    }
}

/// Request to toggle exit node advertisement
#[derive(Deserialize, Debug)]
pub struct ToggleExitNodeRequest {
    /// Enable or disable exit node
    pub enable: bool,
}

/// Response for toggle exit node operation
#[derive(Serialize)]
pub struct ToggleExitNodeResponse {
    /// Whether operation was successful
    pub success: bool,
    /// Response message
    pub message: String,
    /// Current exit node status
    pub exit_node: ExitNodeInfo,
}

/// Exit node information in response
#[derive(Serialize)]
pub struct ExitNodeInfo {
    /// Whether exit node is enabled
    pub enabled: bool,
    /// Whether exit node is advertised
    pub advertised: bool,
    /// Whether exit node is approved
    pub approved: bool,
    /// Number of connected clients
    pub connected_clients: u32,
}

/// Toggle exit node advertisement on/off
pub async fn toggle_exit_node(
    State(state): State<Arc<AppState>>,
    AxumJson(request): AxumJson<ToggleExitNodeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Toggle exit node request: enable={}", request.enable);

    // First check if tailscale container is running
    match state.docker.get_container_status("tailscale").await {
        Ok(container) => {
            let running = container
                .state
                .as_ref()
                .map(|s| s.running.unwrap_or(false))
                .unwrap_or(false);

            if !running {
                return Err(ApiError::Docker(
                    "Tailscale container is not running".to_string(),
                ));
            }
        }
        Err(e) => {
            return Err(ApiError::Docker(format!(
                "Tailscale container not found: {}",
                e
            )));
        }
    }

    let result = if request.enable {
        match state.docker.enable_exit_node().await {
            Ok(_) => {
                state
                    .update_exit_node_state(|s| {
                        s.enabled = true;
                        s.advertised = true;
                    })
                    .await;
                Ok((
                    StatusCode::OK,
                    Json(ToggleExitNodeResponse {
                        success: true,
                        message: "Exit node enabled successfully".to_string(),
                        exit_node: ExitNodeInfo {
                            enabled: true,
                            advertised: true,
                            approved: false,
                            connected_clients: 0,
                        },
                    }),
                ))
            }
            Err(e) => Err(e),
        }
    } else {
        match state.docker.disable_exit_node().await {
            Ok(_) => {
                state
                    .update_exit_node_state(|s| {
                        s.enabled = false;
                        s.advertised = false;
                    })
                    .await;
                Ok((
                    StatusCode::OK,
                    Json(ToggleExitNodeResponse {
                        success: true,
                        message: "Exit node disabled successfully".to_string(),
                        exit_node: ExitNodeInfo {
                            enabled: false,
                            advertised: false,
                            approved: false,
                            connected_clients: 0,
                        },
                    }),
                ))
            }
            Err(e) => Err(e),
        }
    };

    // Update with actual status if successful
    if let Ok((status, mut json_response)) = result {
        match state.docker.get_exit_node_status().await {
            Ok(info) => {
                state
                    .update_exit_node_state(|s| {
                        s.approved = info.approved;
                        s.connected_clients = info.connected_clients;
                    })
                    .await;

                json_response.exit_node.approved = info.approved;
                json_response.exit_node.connected_clients = info.connected_clients;
            }
            Err(e) => {
                warn!("Failed to get updated exit node status: {}", e);
            }
        }
        Ok((status, json_response))
    } else {
        result
    }
}
