//! Rift Protocol Definition
//!
//! Defines the wire protocol for Rift P2P communication.
//! This includes message types for tunnel establishment, data forwarding,
//! and secrets exchange.

use serde::{Deserialize, Serialize};

/// Protocol identifier for Rift
pub const RIFT_PROTOCOL: &str = "/rift/tunnel/1.0.0";

/// Protocol identifier for secrets exchange
pub const RIFT_SECRETS_PROTOCOL: &str = "/rift/secrets/1.0.0";

/// Trait defining the Rift protocol behavior
pub trait RiftProtocol: Send + Sync {
    /// Get the protocol identifier
    fn protocol_id(&self) -> &'static str;

    /// Check if this protocol version is supported
    fn is_supported(&self, version: &str) -> bool;
}

/// Default implementation of Rift protocol
#[derive(Debug, Clone)]
pub struct DefaultProtocol {
    #[allow(dead_code)]
    version: String,
}

impl Default for DefaultProtocol {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
        }
    }
}

impl RiftProtocol for DefaultProtocol {
    fn protocol_id(&self) -> &'static str {
        RIFT_PROTOCOL
    }

    fn is_supported(&self, version: &str) -> bool {
        version == "1.0.0"
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Message Types
// ─────────────────────────────────────────────────────────────────────────────

/// Top-level message envelope for all Rift communications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message ID for tracking/correlation
    pub id: u64,

    /// The actual message payload
    pub payload: MessagePayload,
}

impl Message {
    /// Create a new message with the given payload
    pub fn new(id: u64, payload: MessagePayload) -> Self {
        Self { id, payload }
    }
}

/// All possible message payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    /// Request to establish a tunnel
    TunnelRequest(TunnelRequest),

    /// Response to a tunnel request
    TunnelResponse(TunnelResponse),

    /// Data frame for proxied traffic
    DataFrame(DataFrame),

    /// Secrets exchange request
    SecretsRequest(SecretsRequest),

    /// Secrets exchange response
    SecretsResponse(SecretsResponse),

    /// Connection keepalive/ping
    Ping(PingMessage),

    /// Pong response
    Pong(PongMessage),

    /// Error message
    Error(ErrorMessage),

    /// Graceful close
    Close(CloseMessage),
}

// ─────────────────────────────────────────────────────────────────────────────
// Tunnel Messages
// ─────────────────────────────────────────────────────────────────────────────

/// Request to establish a tunnel to a specific port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelRequest {
    /// Target port on the sharing peer
    pub port: u16,

    /// Requesting peer's public key for secrets encryption (optional)
    pub public_key: Option<Vec<u8>>,

    /// Whether to request secrets
    pub request_secrets: bool,
}

impl TunnelRequest {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            public_key: None,
            request_secrets: false,
        }
    }

    pub fn with_secrets(mut self, public_key: Vec<u8>) -> Self {
        self.public_key = Some(public_key);
        self.request_secrets = true;
        self
    }
}

/// Response to a tunnel request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelResponse {
    /// Whether the tunnel was accepted
    pub accepted: bool,

    /// Stream ID for this tunnel
    pub stream_id: Option<u64>,

    /// Rejection reason if not accepted
    pub reason: Option<String>,
}

impl TunnelResponse {
    pub fn accepted(stream_id: u64) -> Self {
        Self {
            accepted: true,
            stream_id: Some(stream_id),
            reason: None,
        }
    }

    pub fn rejected(reason: String) -> Self {
        Self {
            accepted: false,
            stream_id: None,
            reason: Some(reason),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Data Frames
// ─────────────────────────────────────────────────────────────────────────────

/// Data frame for proxied TCP traffic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFrame {
    /// Stream ID this data belongs to
    pub stream_id: u64,

    /// Sequence number for ordering
    pub sequence: u64,

    /// The actual data payload
    pub data: Vec<u8>,

    /// Whether this is the final frame
    pub fin: bool,
}

impl DataFrame {
    pub fn new(stream_id: u64, sequence: u64, data: Vec<u8>) -> Self {
        Self {
            stream_id,
            sequence,
            data,
            fin: false,
        }
    }

    pub fn with_fin(mut self) -> Self {
        self.fin = true;
        self
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Secrets Messages
// ─────────────────────────────────────────────────────────────────────────────

/// Request for encrypted secrets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsRequest {
    /// Requester's X25519 public key
    pub public_key: Vec<u8>,
}

/// Encrypted secrets response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsResponse {
    /// Responder's ephemeral X25519 public key
    pub ephemeral_public_key: Vec<u8>,

    /// Encrypted env variables (AES-GCM encrypted)
    pub encrypted_data: Vec<u8>,

    /// Nonce used for encryption
    pub nonce: Vec<u8>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Control Messages
// ─────────────────────────────────────────────────────────────────────────────

/// Ping message for keepalive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingMessage {
    pub timestamp: u64,
}

/// Pong response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongMessage {
    pub timestamp: u64,
}

/// Error message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub code: u32,
    pub message: String,
}

impl ErrorMessage {
    pub fn new(code: u32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// Graceful close message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseMessage {
    pub stream_id: Option<u64>,
    pub reason: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Serialization
// ─────────────────────────────────────────────────────────────────────────────

impl Message {
    /// Serialize message to bytes using bincode
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize message from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let request = TunnelRequest::new(3000);
        let message = Message::new(1, MessagePayload::TunnelRequest(request));

        let bytes = message.to_bytes().unwrap();
        let decoded = Message::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.id, 1);
        match decoded.payload {
            MessagePayload::TunnelRequest(req) => {
                assert_eq!(req.port, 3000);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_data_frame() {
        let frame = DataFrame::new(1, 0, vec![1, 2, 3, 4]).with_fin();
        assert!(frame.fin);
        assert_eq!(frame.data, vec![1, 2, 3, 4]);
    }
}
