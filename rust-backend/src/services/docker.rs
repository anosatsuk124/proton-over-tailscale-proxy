use bollard::Docker;
use bollard::container::{
    InspectContainerOptions, ListContainersOptions, LogsOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecOptions};
use bollard::models::ContainerSummary;
use futures::StreamExt;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};
use crate::error::ApiError;
use crate::routes::connection::ConnectRequest;

/// URL used to detect public IP address from inside the container
const PUBLIC_IP_CHECK_URL: &str = "https://ifconfig.me/ip";
/// Container name for the combined ProtonVPN + Tailscale service
pub const CONTAINER_NAME: &str = "proton-tailscale-exit-node";

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
            .start_container(CONTAINER_NAME, None::<StartContainerOptions<String>>)
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
            .start_container(CONTAINER_NAME, None::<StartContainerOptions<String>>)
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
        match self
            .docker
            .stop_container(CONTAINER_NAME, None::<StopContainerOptions>)
            .await
        {
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
        match self
            .docker
            .stop_container(CONTAINER_NAME, None::<StopContainerOptions>)
            .await
        {
            Ok(_) => info!("Tailscale container stopped"),
            Err(e) => {
                if !e.to_string().contains("No such container") {
                    warn!("Failed to stop Tailscale: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Check if the Tailscale exit node is properly advertised
    pub async fn check_exit_node_advertised(&self) -> Result<bool, ApiError> {
        debug!("Checking if exit node is advertised");

        let exec = self
            .docker
            .create_exec(
                CONTAINER_NAME,
                CreateExecOptions {
                    cmd: Some(vec!["tailscale", "status", "--json"]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to create exec: {}", e)))?;

        let mut output = String::new();
        let start_result = self
            .docker
            .start_exec(&exec.id, None::<StartExecOptions>)
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to start exec: {}", e)))?;

        if let bollard::exec::StartExecResults::Attached { output: mut stream, .. } = start_result {
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bollard::container::LogOutput::StdOut { message }) => {
                        output.push_str(&String::from_utf8_lossy(&message));
                    }
                    Ok(bollard::container::LogOutput::StdErr { message }) => {
                        let stderr = String::from_utf8_lossy(&message);
                        warn!("tailscale stderr: {}", stderr);
                    }
                    _ => {}
                }
            }
        }

        match serde_json::from_str::<serde_json::Value>(&output) {
            Ok(json) => {
                let advertised = json["Self"]["ExitNode"].as_bool().unwrap_or(false);
                info!("Exit node advertised: {}", advertised);
                Ok(advertised)
            }
            Err(e) => {
                warn!("Failed to parse tailscale status: {}", e);
                Ok(false)
            }
        }
    }

    /// Get the public IP address from inside the container
    pub async fn get_public_ip(&self) -> Result<Option<String>, ApiError> {
        debug!("Getting public IP from container");

        let exec = self
            .docker
            .create_exec(
                CONTAINER_NAME,
                CreateExecOptions {
                    cmd: Some(vec!["wget", "-qO-", "--timeout=5", PUBLIC_IP_CHECK_URL]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to create exec: {}", e)))?;

        let mut output = String::new();
        let start_result = self
            .docker
            .start_exec(&exec.id, None::<StartExecOptions>)
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to start exec: {}", e)))?;

        if let bollard::exec::StartExecResults::Attached { output: mut stream, .. } = start_result {
            while let Some(chunk) = stream.next().await {
                if let Ok(bollard::container::LogOutput::StdOut { message }) = chunk {
                    output.push_str(&String::from_utf8_lossy(&message));
                }
            }
        }

        let ip = output.trim().to_string();
        if ip.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ip))
        }
    }

    /// Get exit node status from Tailscale
    pub async fn get_exit_node_status(&self) -> Result<ExitNodeInfo, ApiError> {
        debug!("Getting exit node status from Tailscale");

        let exec = self
            .docker
            .create_exec(
                CONTAINER_NAME,
                CreateExecOptions {
                    cmd: Some(vec!["tailscale", "status", "--json"]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to create exec: {}", e)))?;

        let mut output = String::new();
        let start_result = self
            .docker
            .start_exec(&exec.id, None::<StartExecOptions>)
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to start exec: {}", e)))?;

        if let bollard::exec::StartExecResults::Attached { output: mut stream, .. } = start_result {
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bollard::container::LogOutput::StdOut { message }) => {
                        output.push_str(&String::from_utf8_lossy(&message));
                    }
                    Ok(bollard::container::LogOutput::StdErr { message }) => {
                        let stderr = String::from_utf8_lossy(&message);
                        warn!("tailscale stderr: {}", stderr);
                    }
                    _ => {}
                }
            }
        }

        match serde_json::from_str::<serde_json::Value>(&output) {
            Ok(json) => {
                // ExitNodeOption means this node offers itself as an exit node
                let advertised = json["Self"]["ExitNodeOption"].as_bool().unwrap_or(false);
                let approved = json["Self"]["ExitNode"].as_bool().unwrap_or(false)
                    || json["Self"]["Online"].as_bool().unwrap_or(false) && advertised;
                let tailscale_ip = json["Self"]["TailscaleIPs"]
                    .as_array()
                    .and_then(|ips| ips.first())
                    .and_then(|ip| ip.as_str())
                    .map(|s| s.to_string());

                // Peer is an object (map), not an array
                let connected_clients = json["Peer"]
                    .as_object()
                    .map(|peers| {
                        peers
                            .values()
                            .filter(|peer| {
                                // Peer using us as exit node: they have ExitNode set to our key
                                peer["ExitNode"].as_bool().unwrap_or(false)
                            })
                            .count() as u32
                    })
                    .unwrap_or(0);

                // Get public IP in parallel
                let protonvpn_ip = match self.get_public_ip().await {
                    Ok(ip) => ip,
                    Err(e) => {
                        warn!("Failed to get public IP: {}", e);
                        None
                    }
                };

                Ok(ExitNodeInfo {
                    advertised,
                    approved,
                    connected_clients,
                    tailscale_ip,
                    protonvpn_ip,
                })
            }
            Err(e) => Err(ApiError::Docker(format!(
                "Failed to parse tailscale status: {}",
                e
            ))),
        }
    }

    /// Enable exit node advertisement in Tailscale
    pub async fn enable_exit_node(&self) -> Result<(), ApiError> {
        info!("Enabling exit node advertisement");

        // Run tailscale up with advertise-exit-node flag
        let exec = self
            .docker
            .create_exec(
                CONTAINER_NAME,
                CreateExecOptions {
                    cmd: Some(vec!["tailscale", "up", "--advertise-exit-node"]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to create exec: {}", e)))?;

        let start_result = self
            .docker
            .start_exec(&exec.id, None::<StartExecOptions>)
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to start exec: {}", e)))?;

        if let bollard::exec::StartExecResults::Attached { output: mut stream, .. } = start_result {
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bollard::container::LogOutput::StdOut { message }) => {
                        let msg = String::from_utf8_lossy(&message);
                        debug!("tailscale up output: {}", msg);
                    }
                    Ok(bollard::container::LogOutput::StdErr { message }) => {
                        let stderr = String::from_utf8_lossy(&message);
                        if stderr.contains("error") || stderr.contains("Error") {
                            return Err(ApiError::Docker(format!(
                                "Failed to enable exit node: {}",
                                stderr
                            )));
                        }
                        warn!("tailscale up stderr: {}", stderr);
                    }
                    _ => {}
                }
            }
        }

        // Wait a moment for changes to take effect
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Verify exit node is now advertised
        match timeout(Duration::from_secs(10), self.check_exit_node_advertised()).await {
            Ok(Ok(true)) => {
                info!("Exit node successfully enabled");
                Ok(())
            }
            Ok(Ok(false)) => Err(ApiError::Docker(
                "Exit node was not advertised after enabling".to_string(),
            )),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(ApiError::Docker(
                "Timeout waiting for exit node status".to_string(),
            )),
        }
    }

    /// Disable exit node advertisement
    pub async fn disable_exit_node(&self) -> Result<(), ApiError> {
        info!("Disabling exit node advertisement");

        // Run tailscale up without advertise-exit-node flag
        let exec = self
            .docker
            .create_exec(
                CONTAINER_NAME,
                CreateExecOptions {
                    cmd: Some(vec!["tailscale", "up"]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to create exec: {}", e)))?;

        let start_result = self
            .docker
            .start_exec(&exec.id, None::<StartExecOptions>)
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to start exec: {}", e)))?;

        if let bollard::exec::StartExecResults::Attached { output: mut stream, .. } = start_result {
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bollard::container::LogOutput::StdOut { message }) => {
                        let msg = String::from_utf8_lossy(&message);
                        debug!("tailscale up output: {}", msg);
                    }
                    Ok(bollard::container::LogOutput::StdErr { message }) => {
                        let stderr = String::from_utf8_lossy(&message);
                        if stderr.contains("error") || stderr.contains("Error") {
                            return Err(ApiError::Docker(format!(
                                "Failed to disable exit node: {}",
                                stderr
                            )));
                        }
                        warn!("tailscale up stderr: {}", stderr);
                    }
                    _ => {}
                }
            }
        }

        info!("Exit node advertisement disabled");
        Ok(())
    }

    /// Approve exit node in Tailscale (requires admin privileges)
    /// This is typically done via the Tailscale admin console, but we can check approval status
    pub async fn check_exit_node_approval(&self) -> Result<bool, ApiError> {
        debug!("Checking exit node approval status");

        let status = self.get_exit_node_status().await?;
        Ok(status.approved)
    }

    /// Start containers with exit node configuration
    pub async fn start_vpn_with_exit_node(
        &self,
        _request: &ConnectRequest,
        enable_exit_node: bool,
    ) -> Result<(), ApiError> {
        info!("Starting VPN containers with exit_node={}", enable_exit_node);

        // Start Tailscale container first
        match self
            .docker
            .start_container(CONTAINER_NAME, None::<StartContainerOptions<String>>)
            .await
        {
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
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Configure exit node if requested
        if enable_exit_node
            && let Err(e) = self.enable_exit_node().await
        {
            warn!("Failed to enable exit node: {}", e);
            // Continue anyway, VPN will still work without exit node
        }

        // Start ProtonVPN container
        match self
            .docker
            .start_container(CONTAINER_NAME, None::<StartContainerOptions<String>>)
            .await
        {
            Ok(_) => info!("ProtonVPN container started"),
            Err(e) => {
                if e.to_string().contains("No such container") {
                    return Err(ApiError::Docker(
                        "ProtonVPN container not found. Please ensure containers are created."
                            .to_string(),
                    ));
                } else {
                    return Err(ApiError::Docker(format!(
                        "Failed to start ProtonVPN: {}",
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Stop VPN containers, handling exit node cleanup
    pub async fn stop_vpn_with_exit_node(&self) -> Result<(), ApiError> {
        info!("Stopping VPN containers with exit node cleanup");

        // Disable exit node first to prevent connection issues
        if let Err(e) = self.disable_exit_node().await {
            warn!("Failed to disable exit node during shutdown: {}", e);
        }

        // Continue with normal stop
        self.stop_vpn().await
    }

    /// Get environment variables from the running container
    pub async fn get_container_env(&self) -> Result<std::collections::HashMap<String, String>, ApiError> {
        let inspect = self
            .docker
            .inspect_container(CONTAINER_NAME, None::<InspectContainerOptions>)
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to inspect container: {}", e)))?;

        let mut env_map = std::collections::HashMap::new();

        if let Some(config) = inspect.config
            && let Some(env_vars) = config.env
        {
            for var in env_vars {
                if let Some((key, value)) = var.split_once('=') {
                    env_map.insert(key.to_string(), value.to_string());
                }
            }
        }

        Ok(env_map)
    }
}

/// Information about exit node status
#[derive(Debug, Clone)]
pub struct ExitNodeInfo {
    /// Whether exit node is advertised
    pub advertised: bool,
    /// Whether exit node is approved
    pub approved: bool,
    /// Number of connected clients
    pub connected_clients: u32,
    /// Tailscale IP address
    pub tailscale_ip: Option<String>,
    /// ProtonVPN public IP address
    pub protonvpn_ip: Option<String>,
}
