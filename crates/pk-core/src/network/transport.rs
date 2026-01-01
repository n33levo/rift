//! Transport Configuration for PortKey
//!
//! Configures the QUIC transport with noise encryption for secure P2P communication.

use libp2p::{
    quic,
    identity::Keypair,
};
use std::time::Duration;

use crate::error::Result;

/// Default connection timeout
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default idle connection timeout
pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

/// Build the QUIC transport for PortKey
pub fn build_quic_transport(
    keypair: &Keypair,
) -> Result<quic::tokio::Transport> {
    let config = quic::Config::new(keypair);
    
    Ok(quic::tokio::Transport::new(config))
}

/// Transport configuration options
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Connection timeout
    pub connection_timeout: Duration,

    /// Idle connection timeout
    pub idle_timeout: Duration,

    /// Maximum incoming streams per connection
    pub max_incoming_streams: u32,

    /// Maximum outgoing streams per connection  
    pub max_outgoing_streams: u32,

    /// Keep alive interval
    pub keep_alive_interval: Duration,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            connection_timeout: DEFAULT_TIMEOUT,
            idle_timeout: DEFAULT_IDLE_TIMEOUT,
            max_incoming_streams: 256,
            max_outgoing_streams: 256,
            keep_alive_interval: Duration::from_secs(15),
        }
    }
}

impl TransportConfig {
    /// Create a new transport configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set connection timeout
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Set idle timeout
    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Set max incoming streams
    pub fn with_max_incoming_streams(mut self, max: u32) -> Self {
        self.max_incoming_streams = max;
        self
    }

    /// Set keep alive interval
    pub fn with_keep_alive_interval(mut self, interval: Duration) -> Self {
        self.keep_alive_interval = interval;
        self
    }
}
