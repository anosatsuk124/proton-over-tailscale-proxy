use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

pub mod error;
pub mod models;
pub mod routes;
pub mod services;

use error::ApiError;
use models::AppState;
use services::{docker::DockerService, config::ConfigService};

pub struct App {
    state: Arc<AppState>,
}

impl App {
    pub async fn new() -> Result<Self, ApiError> {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();

        let config = ConfigService::load().await?;
        let docker = DockerService::new().await?;

        let state = Arc::new(AppState::new(config, docker));

        Ok(Self { state })
    }
    
    pub async fn run(self) -> Result<(), ApiError> {
        let app = self.create_router();
        
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
        
        info!("Server running on http://0.0.0.0:8080");
        
        axum::serve(listener, app)
            .await?;
        
        Ok(())
    }
    
    fn create_router(&self) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
        
        Router::new()
            // Health check
            .route("/health", get(routes::health::health_check))
            // Status
            .route("/status", get(routes::status::get_status))
            // Connection control
            .route("/connect", post(routes::connection::connect))
            .route("/disconnect", post(routes::connection::disconnect))
            // Exit node control
            .route("/exit-node", post(routes::connection::toggle_exit_node))
            // Action endpoint (frontend uses this)
            .route("/action", post(routes::action::execute_action))
            // Logs
            .route("/logs", get(routes::logs::get_logs))
            .route("/logs/stream", get(routes::logs::stream_logs))
            // Configuration (GET + POST)
            .route("/config", get(routes::config::get_config).post(routes::config::update_config))
            // State
            .with_state(self.state.clone())
            // Middleware
            .layer(TraceLayer::new_for_http())
            .layer(cors)
    }
}
