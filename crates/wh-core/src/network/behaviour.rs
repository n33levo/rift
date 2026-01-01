//! Network Behaviour for Rift
//!
//! Combines libp2p protocols for P2P tunneling:
//! - Identify: Peer info exchange
//! - Ping: Connection liveness  
//! - mDNS: Local network discovery
//! - Relay/DCUtR: NAT traversal
//! - Stream: Raw bidirectional streams for tunnel data

use libp2p::{
    dcutr,
    identify,
    mdns,
    ping,
    relay,
    swarm::NetworkBehaviour,
    StreamProtocol,
};

/// The protocol identifier for Rift tunnel streams
pub const TUNNEL_PROTOCOL: StreamProtocol = StreamProtocol::new("/rift/tunnel/1.0.0");

/// The protocol identifier for Rift secrets exchange
pub const SECRETS_PROTOCOL: StreamProtocol = StreamProtocol::new("/rift/secrets/1.0.0");

/// Combined network behaviour for Rift
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "RiftBehaviourEvent")]
pub struct RiftBehaviour {
    /// Identify protocol for peer info exchange
    pub identify: identify::Behaviour,
    /// Ping protocol for connection liveness
    pub ping: ping::Behaviour,
    /// mDNS for local network discovery
    pub mdns: mdns::tokio::Behaviour,
    /// Relay client for NAT traversal
    pub relay: relay::client::Behaviour,
    /// DCUtR for hole punching
    pub dcutr: dcutr::Behaviour,
    /// Stream behaviour for raw tunnel streams
    pub stream: libp2p_stream::Behaviour,
}

/// Events emitted by the Rift behaviour
#[derive(Debug)]
pub enum RiftBehaviourEvent {
    Identify(identify::Event),
    Ping(ping::Event),
    Mdns(mdns::Event),
    Relay(relay::client::Event),
    Dcutr(dcutr::Event),
    #[allow(dead_code)]
    Stream(()),
}

impl From<identify::Event> for RiftBehaviourEvent {
    fn from(event: identify::Event) -> Self {
        RiftBehaviourEvent::Identify(event)
    }
}

impl From<ping::Event> for RiftBehaviourEvent {
    fn from(event: ping::Event) -> Self {
        RiftBehaviourEvent::Ping(event)
    }
}

impl From<mdns::Event> for RiftBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        RiftBehaviourEvent::Mdns(event)
    }
}

impl From<relay::client::Event> for RiftBehaviourEvent {
    fn from(event: relay::client::Event) -> Self {
        RiftBehaviourEvent::Relay(event)
    }
}

impl From<dcutr::Event> for RiftBehaviourEvent {
    fn from(event: dcutr::Event) -> Self {
        RiftBehaviourEvent::Dcutr(event)
    }
}

impl From<()> for RiftBehaviourEvent {
    fn from(_: ()) -> Self {
        RiftBehaviourEvent::Stream(())
    }
}
