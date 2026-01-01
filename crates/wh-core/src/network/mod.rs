//! P2P Network Module for PortKey
//!
//! Implements the libp2p swarm with QUIC transport, peer discovery,
//! and NAT hole punching capabilities.

pub mod behaviour;
pub mod identity;
pub mod swarm;

pub use behaviour::{PortKeyBehaviour, PortKeyBehaviourEvent, TUNNEL_PROTOCOL, SECRETS_PROTOCOL};
pub use identity::PeerIdentity;
pub use swarm::{
    NetworkEvent, PeerNetwork, PeerInfo, 
    bridge_stream_to_tcp, open_tunnel_stream,
    send_secrets, receive_secrets,
    send_secrets_to_peer, receive_secrets_from_stream,
};
