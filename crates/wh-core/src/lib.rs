//! Rift Core Library
//!
//! This crate provides the core P2P networking and tunneling logic for Rift.
//! It includes:
//! - libp2p swarm management with QUIC transport
//! - Peer discovery and NAT hole punching
//! - TCP proxy tunneling over QUIC streams
//! - EnvVault secrets management

pub mod brand;
pub mod config;
pub mod crypto;
pub mod error;
pub mod network;
pub mod secrets;

pub use config::PortKeyConfig;
pub use error::{PortKeyError, Result};
pub use network::{
    NetworkEvent, PeerNetwork, PeerIdentity,
    bridge_stream_to_tcp, open_tunnel_stream,
    send_secrets, receive_secrets,
    SECRETS_PROTOCOL, TUNNEL_PROTOCOL,
};
pub use secrets::EnvVault;

// Re-export libp2p types we expose
pub use libp2p::{PeerId, Multiaddr, Stream};
pub use libp2p_stream;
