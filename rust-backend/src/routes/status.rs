use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Serialize;
use std::sync::Arc;
use tracing::{debug, error, info};
use crate::{
    error::ApiError,
    models::AppState,
};

/// Response structure for status endpoint
#[derive(Serialize)]
pub struct StatusResponse {
    /// Whether VPN is fully connected (both containers running)
    pub connected: bool,
    /// VPN container status
    pub vpn_container: ContainerStatus,
    /// Tailscale container status
    pub tailscale_container: ContainerStatus,
    /// Exit node status information
    pub exit_node: ExitNodeStatusResponse,
    /// Last error message if any
    pub last_error: Option<String>,
}

/// Container status information
#[derive(Serialize)]
pub struct ContainerStatus {
    /// Container name
    pub name: String,
    /// Whether container is running
    pub running: bool,
    /// Container status string
    pub status: String,
    /// Container image
    pub image: String,
}

/// Exit node status for API response
#[derive(Serialize)]
pub struct ExitNodeStatusResponse {
    /// Whether exit node feature is enabled
    pub enabled: bool,
    /// Whether this node is advertising as exit node
    pub advertised: bool,
    /// Whether exit node is approved by Tailscale admin
    pub approved: bool,
    /// Number of connected clients
    pub connected_clients: u32,
    /// Tailscale IP address
    pub tailscale_ip: Option<String>,
    /// Whether this node is functioning as an exit node
    pub is_exit_node: bool,
}

/// Get current system status including VPN and exit node information
pub async fn get_status(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Fetching system status");

    let vpn_status = state.docker.get_container_status("proton-vpn").await.ok();
    let tailscale_status = state.docker.get_container_status("tailscale").await.ok();

    let vpn_container = match vpn_status {
        Some(s) => {
            let running = s
                .state
                .as_ref()
                .map(|state| state.running.unwrap_or(false))
                .unwrap_or(false);
            let status = s
                .state
                .as_ref()
                .and_then(|state| state.status.as_ref())
                .map(|status| format!("{:?}", status).to_lowercase())
                .unwrap_or_else(|| "unknown".to_string());
            ContainerStatus {
                name: s
                    .name
                    .unwrap_or_default()
                    .trim_start_matches('/')
                    .to_string(),
                running,
                status,
                image: s.image.unwrap_or_default(),
            }
        }
        None => ContainerStatus {
            name: "proton-vpn".to_string(),
            running: false,
            status: "not_found".to_string(),
            image: String::new(),
        },
    };

    let tailscale_container = match tailscale_status {
        Some(s) => {
            let running = s
                .state
                .as_ref()
                .map(|state| state.running.unwrap_or(false))
                .unwrap_or(false);
            let status = s
                .state
                .as_ref()
                .and_then(|state| state.status.as_ref())
                .map(|status| format!("{:?}", status).to_lowercase())
                .unwrap_or_else(|| "unknown".to_string());
            ContainerStatus {
                name: s
                    .name
                    .unwrap_or_default()
                    .trim_start_matches('/')
                    .to_string(),
                running,
                status,
                image: s.image.unwrap_or_default(),
            }
        }
        None => ContainerStatus {
            name: "tailscale".to_string(),
            running: false,
            status: "not_found".to_string(),
            image: String::new(),
        },
    };

    // Get exit node status from app state
    let exit_node_status = state.get_exit_node_status().await;

    // If Tailscale is running, try to get updated exit node status from container
    let exit_node = if tailscale_container.running {
        match state.docker.get_exit_node_status().await {
            Ok(info) => {
                debug!("Retrieved exit node status from Tailscale: {:?}", info);

                // Update stored state with latest info
                state
                    .update_exit_node_state(|state| {
                        state.advertised = info.advertised;
                        state.approved = info.approved;
                        state.connected_clients = info.connected_clients;
                        state.tailscale_ip = info.tailscale_ip.clone();
                    })
                    .await;

                ExitNodeStatusResponse {
                    enabled: exit_node_status.enabled,
                    advertised: info.advertised,
                    approved: info.approved,
                    connected_clients: info.connected_clients,
                    tailscale_ip: info.tailscale_ip,
                    is_exit_node: info.advertised && info.approved,
                }
            }
            Err(e) => {
                error!("Failed to get exit node status from Tailscale: {}", e);
                ExitNodeStatusResponse {
                    enabled: exit_node_status.enabled,
                    advertised: exit_node_status.advertised,
                    approved: exit_node_status.approved,
                    connected_clients: exit_node_status.connected_clients,
                    tailscale_ip: exit_node_status.tailscale_ip,
                    is_exit_node: exit_node_status.is_exit_node,
                }
            }
        }
    } else {
        ExitNodeStatusResponse {
            enabled: exit_node_status.enabled,
            advertised: false,
            approved: false,
            connected_clients: 0,
            tailscale_ip: None,
            is_exit_node: false,
        }
    };

    let response = StatusResponse {
        connected: vpn_container.running && tailscale_container.running,
        vpn_container,
        tailscale_container,
        exit_node,
        last_error: None,
    };

    info!("Status response: connected={}, exit_node={}", response.connected, response.exit_node.is_exit_node);
    Ok((StatusCode::OK, Json(response)))
}
