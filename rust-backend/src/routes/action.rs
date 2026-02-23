use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, error};
use crate::{
    error::ApiError,
    models::AppState,
    routes::status::ApiResponse,
};

#[derive(Deserialize)]
pub struct ActionRequest {
    pub action: String,
}

#[derive(Serialize)]
pub struct ActionMessage {
    pub message: String,
}

pub async fn execute_action(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ActionRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Action request: {}", request.action);

    let message = match request.action.as_str() {
        "enable_exit_node" => {
            state.update_exit_node_state(|s| {
                s.enabled = true;
                s.advertised = true;
            }).await;

            // Try to enable via docker if container exists
            match state.docker.enable_exit_node().await {
                Ok(_) => "Exit node enabled".to_string(),
                Err(e) => {
                    error!("Failed to enable exit node: {}", e);
                    format!("Exit node state updated (container control unavailable: {})", e)
                }
            }
        }
        "disable_exit_node" => {
            state.update_exit_node_state(|s| {
                s.enabled = false;
                s.advertised = false;
            }).await;

            match state.docker.disable_exit_node().await {
                Ok(_) => "Exit node disabled".to_string(),
                Err(e) => {
                    error!("Failed to disable exit node: {}", e);
                    format!("Exit node state updated (container control unavailable: {})", e)
                }
            }
        }
        "approve_exit_node" => {
            // Approval is done via Tailscale admin panel, not API
            "Exit node approval must be done via Tailscale admin panel".to_string()
        }
        "restart" => {
            info!("Restart requested");
            "Restart must be performed via docker compose".to_string()
        }
        _ => {
            return Ok(Json(ApiResponse {
                success: false,
                data: None::<ActionMessage>,
                error: Some(format!("Unknown action: {}", request.action)),
            }));
        }
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(ActionMessage { message }),
        error: None,
    }))
}
