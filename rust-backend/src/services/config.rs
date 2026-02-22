use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use crate::error::ApiError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub vpn: VpnConfig,
    pub tailscale: TailscaleConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnConfig {
    pub enabled: bool,
    pub default_server: String,
    pub protocol: String,
    pub auto_connect: bool,
    pub container_name: String,
    pub image: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailscaleConfig {
    pub enabled: bool,
    pub container_name: String,
    pub image: String,
    pub auth_key_env: String,
    /// Whether to automatically advertise as exit node on connect
    pub advertise_exit_node: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vpn: VpnConfig {
                enabled: true,
                default_server: "us-free-01.protonvpn.net".to_string(),
                protocol: "udp".to_string(),
                auto_connect: false,
                container_name: "proton-vpn".to_string(),
                image: "ghcr.io/tprasadtp/protonvpn".to_string(),
            },
            tailscale: TailscaleConfig {
                enabled: true,
                container_name: "tailscale".to_string(),
                image: "tailscale/tailscale".to_string(),
                auth_key_env: "TAILSCALE_AUTHKEY".to_string(),
                advertise_exit_node: false,
            },
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
            },
        }
    }
}

pub struct ConfigService;

impl ConfigService {
    pub async fn load() -> Result<Config, ApiError> {
        let config_paths = [
            PathBuf::from("/etc/proton-vpn-api/config.json"),
            PathBuf::from("config.json"),
            PathBuf::from("config.yaml"),
            PathBuf::from("config.yml"),
            PathBuf::from("config.toml"),
        ];
        
        for path in &config_paths {
            if path.exists() {
                info!("Loading configuration from {:?}", path);
                let content = fs::read_to_string(path).await?;
                
                let config = if path.extension().map(|e| e == "json").unwrap_or(false) {
                    serde_json::from_str(&content)
                        .map_err(|e| ApiError::Config(format!("Invalid JSON config: {}", e)))?
                } else if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
                    serde_yaml::from_str(&content)
                        .map_err(|e| ApiError::Config(format!("Invalid YAML config: {}", e)))?
                } else if path.extension().map(|e| e == "toml").unwrap_or(false) {
                    toml::from_str(&content)
                        .map_err(|e| ApiError::Config(format!("Invalid TOML config: {}", e)))?
                } else {
                    Config::default()
                };
                
                return Ok(config);
            }
        }
        
        info!("No configuration file found, using defaults");
        Ok(Config::default())
    }
    
    pub async fn save(config: &Config, path: &PathBuf) -> Result<(), ApiError> {
        let content = serde_json::to_string_pretty(config)?;
        fs::write(path, content).await?;
        info!("Configuration saved to {:?}", path);
        Ok(())
    }
}
