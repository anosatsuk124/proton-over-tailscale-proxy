use bollard::Docker;
use bollard::container::{
    InspectContainerOptions,
    ListContainersOptions,
    LogsOptions,
    StartContainerOptions,
    StopContainerOptions,
};
use bollard::models::ContainerSummary;
use futures::StreamExt;
use std::sync::Arc;
use tracing::{error, info, warn};
use crate::error::ApiError;
use crate::routes::connection::ConnectRequest;

pub struct DockerService {
    docker: Docker,
}

impl DockerService {
    pub async fn new() -> Result<Arc<Self>, ApiError> {
        let docker = Docker::connect_with_defaults()
            .map_err(|e| ApiError::Docker(format!("Failed to connect to Docker: {}", e)))?;
        
        info!("Connected to Docker daemon");
        
        Ok(Arc::new(Self { docker }))
    }
    
    pub async fn get_container_status(
        &self,
        container_name: &str,
    ) -> Result<bollard::models::ContainerInspectResponse, ApiError> {
        self.docker
            .inspect_container(container_name, None::<InspectContainerOptions>)
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to inspect container: {}", e)))
    }
    
    pub async fn list_containers(&self) -> Result<Vec<ContainerSummary>, ApiError> {
        let options = Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        });
        
        self.docker
            .list_containers(options)
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to list containers: {}", e)))
    }
    
    pub async fn get_container_logs(
        &self,
        container_name: &str,
        lines: usize,
    ) -> Result<Vec<String>, ApiError> {
        let options = Some(LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: format!("{}", lines),
            timestamps: true,
            ..Default::default()
        });
        
        let mut logs = Vec::new();
        let mut stream = self.docker.logs(container_name, options);
        
        while let Some(log_result) = stream.next().await {
            match log_result {
                Ok(log) => {
                    let log_str = log.to_string();
                    logs.push(log_str);
                }
                Err(e) => {
                    error!("Error reading log: {}", e);
                }
            }
        }
        
        Ok(logs)
    }
    
    pub async fn start_vpn(&self, _request: &ConnectRequest) -> Result<(), ApiError> {
        info!("Starting VPN containers");
        
        // Start Tailscale container first
        match self.docker
            .start_container("tailscale", None::<StartContainerOptions<String>>)
            .await {
            Ok(_) => info!("Tailscale container started"),
            Err(e) => {
                if e.to_string().contains("No such container") {
                    warn!("Tailscale container not found, may need to be created");
                } else {
                    error!("Failed to start tailscale: {}", e);
                }
            }
        }
        
        // Wait a moment for Tailscale to initialize
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Start ProtonVPN container
        match self.docker
            .start_container("proton-vpn", None::<StartContainerOptions<String>>)
            .await {
            Ok(_) => info!("ProtonVPN container started"),
            Err(e) => {
                if e.to_string().contains("No such container") {
                    return Err(ApiError::Docker(
                        "ProtonVPN container not found. Please ensure containers are created.".to_string()
                    ));
                } else {
                    return Err(ApiError::Docker(format!("Failed to start ProtonVPN: {}", e)));
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn stop_vpn(&self) -> Result<(), ApiError> {
        info!("Stopping VPN containers");
        
        // Stop ProtonVPN first
        match self.docker
            .stop_container("proton-vpn", None::<StopContainerOptions>)
            .await {
            Ok(_) => info!("ProtonVPN container stopped"),
            Err(e) => {
                if !e.to_string().contains("No such container") {
                    warn!("Failed to stop ProtonVPN: {}", e);
                }
            }
        }
        
        // Wait for VPN to stop
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Stop Tailscale
        match self.docker
            .stop_container("tailscale", None::<StopContainerOptions>)
            .await {
            Ok(_) => info!("Tailscale container stopped"),
            Err(e) => {
                if !e.to_string().contains("No such container") {
                    warn!("Failed to stop Tailscale: {}", e);
                }
            }
        }
        
        Ok(())
    }
}
