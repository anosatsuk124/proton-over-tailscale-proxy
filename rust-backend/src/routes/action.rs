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

    match request.action.as_str() {
        "enable_exit_node" => {
            match state.docker.enable_exit_node().await {
                Ok(_) => {
                    state.update_exit_node_state(|s| {
                        s.enabled = true;
                        s.advertised = true;
                    }).await;
                    Ok(Json(ApiResponse {
                        success: true,
                        data: Some(ActionMessage { message: "Exit node enabled".to_string() }),
                        error: None,
                    }))
                }
                Err(e) => {
                    error!("Failed to enable exit node: {}", e);
                    Ok(Json(ApiResponse {
                        success: false,
                        data: None::<ActionMessage>,
                        error: Some(format!("Failed to enable exit node: {}", e)),
                    }))
                }
            }
        }
        "disable_exit_node" => {
            match state.docker.disable_exit_node().await {
                Ok(_) => {
                    state.update_exit_node_state(|s| {
                        s.enabled = false;
                        s.advertised = false;
                    }).await;
                    Ok(Json(ApiResponse {
                        success: true,
                        data: Some(ActionMessage { message: "Exit node disabled".to_string() }),
                        error: None,
                    }))
                }
                Err(e) => {
                    error!("Failed to disable exit node: {}", e);
                    Ok(Json(ApiResponse {
                        success: false,
                        data: None::<ActionMessage>,
                        error: Some(format!("Failed to disable exit node: {}", e)),
                    }))
                }
            }
        }
        "approve_exit_node" => {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(ActionMessage { message: "Exit node approval must be done via Tailscale admin panel".to_string() }),
                error: None,
            }))
        }
        "restart" => {
            info!("Restart requested");
            match state.docker.restart_container().await {
                Ok(_) => {
                    Ok(Json(ApiResponse {
                        success: true,
                        data: Some(ActionMessage { message: "Container restarted".to_string() }),
                        error: None,
                    }))
                }
                Err(e) => {
                    error!("Failed to restart container: {}", e);
                    Ok(Json(ApiResponse {
                        success: false,
                        data: None::<ActionMessage>,
                        error: Some(format!("Failed to restart container: {}", e)),
                    }))
                }
            }
        }
        _ => {
            Ok(Json(ApiResponse {
                success: false,
                data: None::<ActionMessage>,
                error: Some(format!("Unknown action: {}", request.action)),
            }))
        }
    }
}
