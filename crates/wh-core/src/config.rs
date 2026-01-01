//! Configuration management for Rift
//!
//! Handles loading and saving of Rift configuration including
//! identity keys, known peers, and user preferences.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{RiftError, Result};

/// Main configuration for Rift
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiftConfig {
    /// Path to the identity keypair file
    pub identity_path: PathBuf,

    /// Port for the local QUIC listener (0 = random)
    pub listen_port: u16,

    /// Enable mDNS for local network peer discovery
    pub enable_mdns: bool,

    /// Enable relay client for NAT traversal
    pub enable_relay: bool,

    /// Bootstrap peers for initial discovery
    pub bootstrap_peers: Vec<String>,

    /// Rendezvous server address (optional)
    pub rendezvous_server: Option<String>,

    /// Maximum number of concurrent connections
    pub max_connections: usize,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,

    /// Enable debug logging
    pub debug: bool,
}

impl Default for RiftConfig {
    fn default() -> Self {
        Self {
            identity_path: Self::default_identity_path(),
            listen_port: 0,
            enable_mdns: true,
            enable_relay: true,
            // Public IPFS relays for testing (use sparingly!)
            bootstrap_peers: vec![
                "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN".to_string(),
                "/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa".to_string(),
            ],
            rendezvous_server: None,
            max_connections: 64,
            connection_timeout_secs: 30,
            debug: false,
        }
    }
}

impl RiftConfig {
    /// Creates a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads configuration from a file
    pub fn load(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        toml::from_str(&contents).map_err(|e| RiftError::ConfigError(e.to_string()))
    }

    /// Saves configuration to a file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| RiftError::ConfigError(e.to_string()))?;
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Returns the default configuration directory
    pub fn default_config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rift")
    }

    /// Returns the default configuration file path
    pub fn default_config_path() -> PathBuf {
        Self::default_config_dir().join("config.toml")
    }

    /// Returns the default identity keypair path
    pub fn default_identity_path() -> PathBuf {
        Self::default_config_dir().join("identity.key")
    }

    /// Builder pattern: set listen port
    pub fn with_listen_port(mut self, port: u16) -> Self {
        self.listen_port = port;
        self
    }

    /// Builder pattern: enable/disable mDNS
    pub fn with_mdns(mut self, enable: bool) -> Self {
        self.enable_mdns = enable;
        self
    }

    /// Builder pattern: enable/disable relay
    pub fn with_relay(mut self, enable: bool) -> Self {
        self.enable_relay = enable;
        self
    }

    /// Builder pattern: set rendezvous server
    pub fn with_rendezvous_server(mut self, server: String) -> Self {
        self.rendezvous_server = Some(server);
        self
    }

    /// Builder pattern: add bootstrap peer
    pub fn with_bootstrap_peer(mut self, peer: String) -> Self {
        self.bootstrap_peers.push(peer);
        self
    }

    /// Builder pattern: set debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RiftConfig::default();
        assert_eq!(config.listen_port, 0);
        assert!(config.enable_mdns);
        assert!(config.enable_relay);
    }

    #[test]
    fn test_builder_pattern() {
        let config = RiftConfig::new()
            .with_listen_port(8080)
            .with_mdns(false)
            .with_debug(true);

        assert_eq!(config.listen_port, 8080);
        assert!(!config.enable_mdns);
        assert!(config.debug);
    }
}
