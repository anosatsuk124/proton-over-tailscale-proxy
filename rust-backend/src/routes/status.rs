use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Serialize;
use std::sync::Arc;
use crate::{
    error::ApiError,
    models::AppState,
};

#[derive(Serialize)]
pub struct StatusResponse {
    pub connected: bool,
    pub vpn_container: ContainerStatus,
    pub tailscale_container: ContainerStatus,
    pub last_error: Option<String>,
}

#[derive(Serialize)]
pub struct ContainerStatus {
    pub name: String,
    pub running: bool,
    pub status: String,
    pub image: String,
}

pub async fn get_status(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let vpn_status = state.docker.get_container_status("proton-vpn").await.ok();
    let tailscale_status = state.docker.get_container_status("tailscale").await.ok();
    
    let vpn_container = match vpn_status {
        Some(s) => {
            let running = s.state.as_ref()
                .map(|state| state.running.unwrap_or(false))
                .unwrap_or(false);
            let status = s.state.as_ref()
                .and_then(|state| state.status.as_ref())
                .map(|status| format!("{:?}", status).to_lowercase())
                .unwrap_or_else(|| "unknown".to_string());
            ContainerStatus {
                name: s.name.unwrap_or_default().trim_start_matches('/').to_string(),
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
            let running = s.state.as_ref()
                .map(|state| state.running.unwrap_or(false))
                .unwrap_or(false);
            let status = s.state.as_ref()
                .and_then(|state| state.status.as_ref())
                .map(|status| format!("{:?}", status).to_lowercase())
                .unwrap_or_else(|| "unknown".to_string());
            ContainerStatus {
                name: s.name.unwrap_or_default().trim_start_matches('/').to_string(),
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
    
    let response = StatusResponse {
        connected: vpn_container.running && tailscale_container.running,
        vpn_container,
        tailscale_container,
        last_error: None,
    };
    
    Ok((StatusCode::OK, Json(response)))
}
