use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use serde::Serialize;
use std::sync::Arc;
use tracing::{debug, error, info};
use crate::{
    error::ApiError,
    models::AppState,
};

/// API response wrapper matching frontend's ApiResponse<T>
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Response structure matching frontend's SystemStatus interface
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    pub protonvpn: String,
    pub tailscale: String,
    pub exit_node: String,
    pub exit_node_enabled: bool,
    pub exit_node_approved: bool,
    pub connected_clients: u32,
    pub tailscale_ip: Option<String>,
    pub protonvpn_ip: Option<String>,
    pub connection_quality: String,
    pub last_updated: String,
}

/// Get current system status matching frontend's SystemStatus interface
pub async fn get_status(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Fetching system status");

    // Check the combined container (runs both ProtonVPN WireGuard and Tailscale)
    let container = state.docker.get_container_status(crate::services::docker::CONTAINER_NAME).await.ok();

    let container_running = container
        .as_ref()
        .and_then(|s| s.state.as_ref())
        .map(|state| state.running.unwrap_or(false))
        .unwrap_or(false);

    // Both services run in the same container
    let vpn_running = container_running;
    let tailscale_running = container_running;

    let protonvpn_status = if vpn_running { "connected" } else { "disconnected" };
    let tailscale_status = if tailscale_running { "connected" } else { "disconnected" };

    // Get exit node status
    let exit_node_status = state.get_exit_node_status().await;

    let exit_node_str = if exit_node_status.advertised && exit_node_status.approved {
        "approved"
    } else if exit_node_status.advertised {
        "advertised"
    } else {
        "not_advertised"
    };

    // Try to get updated info from docker if running
    let (tailscale_ip, exit_advertised, exit_approved, connected_clients_count, protonvpn_ip) = if tailscale_running {
        match state.docker.get_exit_node_status().await {
            Ok(info) => {
                debug!("Retrieved exit node status from Tailscale: {:?}", info);
                state.update_exit_node_state(|s| {
                    s.advertised = info.advertised;
                    s.approved = info.approved;
                    s.connected_clients = info.connected_clients;
                    s.tailscale_ip = info.tailscale_ip.clone();
                }).await;
                (info.tailscale_ip, info.advertised, info.approved, info.connected_clients, info.protonvpn_ip)
            }
            Err(e) => {
                error!("Failed to get exit node status: {}", e);
                (exit_node_status.tailscale_ip, exit_node_status.advertised, exit_node_status.approved, exit_node_status.connected_clients, None)
            }
        }
    } else {
        (None, false, false, 0, None)
    };

    let exit_node_str_updated = if exit_advertised && exit_approved {
        "approved"
    } else if exit_advertised {
        "advertised"
    } else {
        exit_node_str
    };

    let status = SystemStatus {
        protonvpn: protonvpn_status.to_string(),
        tailscale: tailscale_status.to_string(),
        exit_node: exit_node_str_updated.to_string(),
        exit_node_enabled: exit_node_status.enabled || exit_advertised,
        exit_node_approved: exit_approved,
        connected_clients: connected_clients_count,
        tailscale_ip,
        protonvpn_ip,
        connection_quality: if vpn_running && tailscale_running { "good" } else { "unknown" }.to_string(),
        last_updated: chrono::Utc::now().to_rfc3339(),
    };

    info!("Status: vpn={}, tailscale={}, exit_node={}", protonvpn_status, tailscale_status, exit_node_str_updated);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(status),
        error: None,
    }))
}
