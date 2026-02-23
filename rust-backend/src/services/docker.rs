use bollard::Docker;
use bollard::container::{
    InspectContainerOptions, ListContainersOptions, LogsOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecOptions};
use bollard::models::ContainerSummary;
use futures::StreamExt;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{debug, error, info, warn};
use crate::error::ApiError;
use crate::routes::connection::ConnectRequest;

/// Base URL for the internal API server running inside the container
/// Since rust-backend shares the network namespace (network_mode: service:proton-tailscale-exit),
/// the internal API is accessible via localhost.
const CONTAINER_API_BASE: &str = "http://localhost:8081";

/// URL used to detect public IP address from inside the container
const PUBLIC_IP_CHECK_URL: &str = "https://ifconfig.me/ip";

/// Docker Compose label used to find the exit node container.
const COMPOSE_SERVICE_LABEL: &str = "com.docker.compose.service";
const COMPOSE_SERVICE_NAME: &str = "proton-tailscale-exit";

pub struct DockerService {
    docker: Docker,
    container_name: String,
}

impl DockerService {
    pub async fn new() -> Result<Arc<Self>, ApiError> {
        let docker = Docker::connect_with_defaults()
            .map_err(|e| ApiError::Docker(format!("Failed to connect to Docker: {}", e)))?;

        let mut service = Self {
            docker,
            container_name: String::new(),
        };

        // Resolve actual container name from Docker Compose labels
        let resolved = service.resolve_container_name().await?;
        service.container_name = resolved;
        info!("Connected to Docker daemon, exit node container: {}", service.container_name);

        Ok(Arc::new(service))
    }

    /// Resolve the actual container name by searching for the Docker Compose service label.
    async fn resolve_container_name(&self) -> Result<String, ApiError> {
        // Check environment variable override first
        if let Ok(name) = std::env::var("EXIT_NODE_CONTAINER_NAME") {
            info!("Using container name from EXIT_NODE_CONTAINER_NAME: {}", name);
            return Ok(name);
        }

        // Search by Docker Compose service label
        let mut filters = std::collections::HashMap::new();
        filters.insert(
            "label".to_string(),
            vec![format!("{}={}", COMPOSE_SERVICE_LABEL, COMPOSE_SERVICE_NAME)],
        );

        let options = Some(ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        });

        let containers = self
            .docker
            .list_containers(options)
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to list containers: {}", e)))?;

        if let Some(container) = containers.first() {
            if let Some(names) = &container.names {
                if let Some(name) = names.first() {
                    // Docker prefixes container names with "/"
                    let name = name.trim_start_matches('/').to_string();
                    info!("Discovered exit node container: {}", name);
                    return Ok(name);
                }
            }
        }

        // Fallback to the compose service name
        warn!(
            "Could not discover container by label, falling back to '{}'",
            COMPOSE_SERVICE_NAME
        );
        Ok(COMPOSE_SERVICE_NAME.to_string())
    }

    /// Get the resolved container name
    pub fn container_name(&self) -> &str {
        &self.container_name
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
            .start_container(&self.container_name, None::<StartContainerOptions<String>>)
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
            .start_container(&self.container_name, None::<StartContainerOptions<String>>)
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
            .stop_container(&self.container_name, None::<StopContainerOptions>)
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
            .stop_container(&self.container_name, None::<StopContainerOptions>)
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
                &self.container_name,
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
                &self.container_name,
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
                &self.container_name,
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
                // Check AllowedIPs to determine if the exit node is approved by admin
                // When approved, AllowedIPs includes "0.0.0.0/0" and "::/0"
                let allowed_ips = json["Self"]["AllowedIPs"]
                    .as_array()
                    .map(|ips| {
                        ips.iter()
                            .filter_map(|ip| ip.as_str())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let approved = advertised
                    && (allowed_ips.contains(&"0.0.0.0/0") || allowed_ips.contains(&"::/0"));
                let tailscale_ip = json["Self"]["TailscaleIPs"]
                    .as_array()
                    .and_then(|ips| ips.first())
                    .and_then(|ip| ip.as_str())
                    .map(|s| s.to_string());

                // Peer is an object (map), not an array
                // Count active peers as connected clients
                let connected_clients = json["Peer"]
                    .as_object()
                    .map(|peers| {
                        peers
                            .values()
                            .filter(|peer| {
                                peer["Active"].as_bool().unwrap_or(false)
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

    /// Enable exit node advertisement in Tailscale via container internal API
    pub async fn enable_exit_node(&self) -> Result<(), ApiError> {
        info!("Enabling exit node advertisement via container API");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| ApiError::Docker(format!("Failed to create HTTP client: {}", e)))?;

        let resp = client
            .post(format!("{}/exit-node/enable", CONTAINER_API_BASE))
            .send()
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to reach container API: {}", e)))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let err_msg = body["error"].as_str().unwrap_or("Unknown error");
            return Err(ApiError::Docker(format!("Failed to enable exit node: {}", err_msg)));
        }

        info!("Exit node successfully enabled");
        Ok(())
    }

    /// Disable exit node advertisement via container internal API
    pub async fn disable_exit_node(&self) -> Result<(), ApiError> {
        info!("Disabling exit node advertisement via container API");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| ApiError::Docker(format!("Failed to create HTTP client: {}", e)))?;

        let resp = client
            .post(format!("{}/exit-node/disable", CONTAINER_API_BASE))
            .send()
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to reach container API: {}", e)))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let err_msg = body["error"].as_str().unwrap_or("Unknown error");
            return Err(ApiError::Docker(format!("Failed to disable exit node: {}", err_msg)));
        }

        info!("Exit node advertisement disabled");
        Ok(())
    }

    /// Restart the VPN container
    pub async fn restart_container(&self) -> Result<(), ApiError> {
        info!("Restarting container: {}", &self.container_name);

        self.docker
            .restart_container(&self.container_name, None)
            .await
            .map_err(|e| ApiError::Docker(format!("Failed to restart container: {}", e)))?;

        info!("Container restarted: {}", &self.container_name);
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
            .start_container(&self.container_name, None::<StartContainerOptions<String>>)
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
            .start_container(&self.container_name, None::<StartContainerOptions<String>>)
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
            .inspect_container(&self.container_name, None::<InspectContainerOptions>)
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
