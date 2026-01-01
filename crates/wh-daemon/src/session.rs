//! Session Management for Rift
//!
//! Manages sharing and connecting sessions with the stream-based tunnel.

use wh_core::{EnvVault, Result, secrets::SecretsResponse};
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tracing::info;

/// Statistics for a tunnel session
#[derive(Debug, Default)]
pub struct TunnelStats {
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,
    pub active_connections: AtomicU64,
}

impl TunnelStats {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
}

/// Represents an active sharing session (host side)
pub struct ShareSession {
    /// Port being shared
    port: u16,

    /// Secrets file path (optional)
    #[allow(dead_code)]
    secrets_path: Option<PathBuf>,

    /// EnvVault for secrets management
    env_vault: Option<EnvVault>,

    /// Session statistics
    stats: Arc<TunnelStats>,

    /// Session active flag
    active: bool,
}

impl ShareSession {
    /// Create a new sharing session
    pub fn new(port: u16, secrets_path: Option<PathBuf>) -> Result<Self> {
        let env_vault = if let Some(ref path) = secrets_path {
            let mut vault = EnvVault::new();
            vault.load_env_file(path)?;
            info!("Loaded {} secrets from {}", vault.len(), path.display());
            Some(vault)
        } else {
            None
        };

        Ok(Self {
            port,
            secrets_path,
            env_vault,
            stats: TunnelStats::new(),
            active: true,
        })
    }

    /// Get the port being shared
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Check if secrets are available
    pub fn has_secrets(&self) -> bool {
        self.env_vault.is_some()
    }

    /// Get the number of secrets
    pub fn secrets_count(&self) -> usize {
        self.env_vault.as_ref().map(|v| v.len()).unwrap_or(0)
    }

    /// Handle a secrets request from a peer
    pub fn encrypt_secrets_for_peer(&self, peer_public_key: &[u8]) -> Result<Option<SecretsResponse>> {
        if let Some(ref vault) = self.env_vault {
            Ok(Some(vault.encrypt_for_peer(peer_public_key)?))
        } else {
            Ok(None)
        }
    }

    /// Get session statistics
    pub fn stats(&self) -> Arc<TunnelStats> {
        Arc::clone(&self.stats)
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Stop the session
    pub fn stop(&mut self) {
        info!("Stopping share session for port {}", self.port);
        self.active = false;
    }
}

/// Represents an active connecting session (client side)
pub struct ConnectSession {
    /// Remote peer link
    peer_link: String,

    /// Remote port to connect to
    remote_port: u16,

    /// Local port to listen on
    local_port: u16,

    /// EnvVault for receiving secrets
    env_vault: EnvVault,

    /// Session statistics
    stats: Arc<TunnelStats>,

    /// Session active flag
    active: bool,

    /// Whether secrets were received
    secrets_received: bool,
}

impl ConnectSession {
    /// Create a new connecting session
    pub fn new(
        peer_link: String,
        remote_port: u16,
        local_port: Option<u16>,
    ) -> Result<Self> {
        // Use same port locally if not specified
        let local_port = local_port.unwrap_or(remote_port);
        let env_vault = EnvVault::new();

        Ok(Self {
            peer_link,
            remote_port,
            local_port,
            env_vault,
            stats: TunnelStats::new(),
            active: true,
            secrets_received: false,
        })
    }

    /// Get the remote peer link
    pub fn peer_link(&self) -> &str {
        &self.peer_link
    }

    /// Get the remote port
    pub fn remote_port(&self) -> u16 {
        self.remote_port
    }

    /// Get the local port
    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    /// Handle received secrets
    pub fn receive_secrets(&mut self, response: &SecretsResponse) -> Result<()> {
        let secrets = self.env_vault.decrypt_from_peer(response)?;
        
        info!("Received {} secrets from peer", secrets.len());
        
        // Store in our vault
        for (key, value) in secrets {
            self.env_vault.set(key, value);
        }
        
        self.secrets_received = true;
        Ok(())
    }

    /// Inject secrets into environment
    pub fn inject_secrets(&self) {
        if self.secrets_received {
            self.env_vault.inject_into_env();
            info!("Injected {} secrets into environment", self.env_vault.len());
        }
    }

    /// Write secrets to a temp file
    pub fn write_secrets_to_file(&self) -> Result<Option<PathBuf>> {
        if self.secrets_received && !self.env_vault.is_empty() {
            Ok(Some(self.env_vault.write_to_temp_file()?))
        } else {
            Ok(None)
        }
    }

    /// Check if secrets were received
    pub fn has_secrets(&self) -> bool {
        self.secrets_received
    }

    /// Get the number of secrets
    pub fn secrets_count(&self) -> usize {
        self.env_vault.len()
    }

    /// Get session statistics
    pub fn stats(&self) -> Arc<TunnelStats> {
        Arc::clone(&self.stats)
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Stop the session
    pub fn stop(&mut self) {
        info!("Stopping connect session to {}", self.peer_link);
        self.active = false;
    }
}
