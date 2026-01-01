//! Error types for Rift
//!
//! Provides a unified error handling strategy using thiserror.

use thiserror::Error;

/// Result type alias for Rift operations
pub type Result<T> = std::result::Result<T, RiftError>;

/// Unified error type for all Rift operations
#[derive(Error, Debug)]
pub enum RiftError {
    // ─────────────────────────────────────────────────────────────
    // Network Errors
    // ─────────────────────────────────────────────────────────────
    #[error("Failed to initialize P2P network: {0}")]
    NetworkInitialization(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Connection failed to peer {peer_id}: {reason}")]
    ConnectionFailed { peer_id: String, reason: String },

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("Transport error: {0}")]
    TransportError(String),

    #[error("Dial error: {0}")]
    DialError(String),

    // ─────────────────────────────────────────────────────────────
    // Protocol Errors
    // ─────────────────────────────────────────────────────────────
    #[error("Protocol negotiation failed: {0}")]
    ProtocolNegotiation(String),

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(u32),

    // ─────────────────────────────────────────────────────────────
    // Proxy/Tunnel Errors
    // ─────────────────────────────────────────────────────────────
    #[error("Failed to bind to port {port}: {reason}")]
    PortBindFailed { port: u16, reason: String },

    #[error("Tunnel not established")]
    TunnelNotEstablished,

    #[error("Proxy forwarding error: {0}")]
    ProxyForwarding(String),

    #[error("Proxy error: {0}")]
    ProxyError(String),

    // ─────────────────────────────────────────────────────────────
    // Secrets/Crypto Errors
    // ─────────────────────────────────────────────────────────────
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Keyring access failed: {0}")]
    KeyringError(String),

    #[error("Failed to parse env file: {0}")]
    EnvParseError(String),

    // ─────────────────────────────────────────────────────────────
    // Configuration Errors
    // ─────────────────────────────────────────────────────────────
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid peer ID format: {0}")]
    InvalidPeerId(String),

    // ─────────────────────────────────────────────────────────────
    // IO Errors
    // ─────────────────────────────────────────────────────────────
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<libp2p::noise::Error> for RiftError {
    fn from(err: libp2p::noise::Error) -> Self {
        RiftError::EncryptionFailed(err.to_string())
    }
}

impl From<libp2p::TransportError<std::io::Error>> for RiftError {
    fn from(err: libp2p::TransportError<std::io::Error>) -> Self {
        RiftError::TransportError(err.to_string())
    }
}

impl From<serde_json::Error> for RiftError {
    fn from(err: serde_json::Error) -> Self {
        RiftError::Serialization(err.to_string())
    }
}

impl From<bincode::Error> for RiftError {
    fn from(err: bincode::Error) -> Self {
        RiftError::Serialization(err.to_string())
    }
}
