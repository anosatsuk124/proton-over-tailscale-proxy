use crate::services::{config::Config, docker::DockerService};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Application state shared across all routes
pub struct AppState {
    pub config: Config,
    pub docker: Arc<DockerService>,
    pub exit_node_state: Arc<RwLock<ExitNodeState>>,
}

impl AppState {
    /// Create a new AppState instance
    pub fn new(config: Config, docker: Arc<DockerService>) -> Self {
        Self {
            config,
            docker,
            exit_node_state: Arc::new(RwLock::new(ExitNodeState::default())),
        }
    }

    /// Get the current exit node status
    pub async fn get_exit_node_status(&self) -> ExitNodeStatus {
        let state = self.exit_node_state.read().await;
        ExitNodeStatus {
            enabled: state.enabled,
            advertised: state.advertised,
            approved: state.approved,
            connected_clients: state.connected_clients,
            tailscale_ip: state.tailscale_ip.clone(),
            is_exit_node: state.advertised && state.approved,
        }
    }

    /// Update the exit node state
    pub async fn update_exit_node_state<F>(&self, f: F)
    where
        F: FnOnce(&mut ExitNodeState),
    {
        let mut state = self.exit_node_state.write().await;
        f(&mut state);
    }
}

/// Internal state for exit node management
#[derive(Debug, Clone, Default)]
pub struct ExitNodeState {
    /// Whether exit node feature is enabled
    pub enabled: bool,
    /// Whether the exit node is approved by Tailscale admin
    pub approved: bool,
    /// Whether this node is currently advertising as an exit node
    pub advertised: bool,
    /// Number of connected clients using this exit node
    pub connected_clients: u32,
    /// Tailscale IP address of this node
    pub tailscale_ip: Option<String>,
}

/// Serializable status response for exit node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitNodeStatus {
    /// Whether exit node feature is enabled
    pub enabled: bool,
    /// Whether this node is advertising as exit node
    pub advertised: bool,
    /// Whether the exit node is approved by admin
    pub approved: bool,
    /// Number of connected clients
    pub connected_clients: u32,
    /// Tailscale IP address
    pub tailscale_ip: Option<String>,
    /// Whether this node is functioning as an exit node
    pub is_exit_node: bool,
}
