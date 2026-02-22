use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use serde::Serialize;
use std::sync::Arc;
use crate::{
    error::ApiError,
    models::AppState,
};

#[derive(Serialize)]
pub struct ConfigResponse {
    pub vpn_enabled: bool,
    pub tailscale_enabled: bool,
    pub default_server: String,
    pub default_protocol: String,
    pub auto_connect: bool,
}

pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let config = &state.config;
    
    let response = ConfigResponse {
        vpn_enabled: config.vpn.enabled,
        tailscale_enabled: config.tailscale.enabled,
        default_server: config.vpn.default_server.clone(),
        default_protocol: config.vpn.protocol.clone(),
        auto_connect: config.vpn.auto_connect,
    };
    
    Ok(Json(response))
}
