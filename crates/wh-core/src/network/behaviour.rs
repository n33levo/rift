//! Network Behaviour for PortKey
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

/// The protocol identifier for PortKey tunnel streams
pub const TUNNEL_PROTOCOL: StreamProtocol = StreamProtocol::new("/portkey/tunnel/1.0.0");

/// The protocol identifier for PortKey secrets exchange
pub const SECRETS_PROTOCOL: StreamProtocol = StreamProtocol::new("/portkey/secrets/1.0.0");

/// Combined network behaviour for PortKey
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "PortKeyBehaviourEvent")]
pub struct PortKeyBehaviour {
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

/// Events emitted by the PortKey behaviour
#[derive(Debug)]
pub enum PortKeyBehaviourEvent {
    Identify(identify::Event),
    Ping(ping::Event),
    Mdns(mdns::Event),
    Relay(relay::client::Event),
    Dcutr(dcutr::Event),
    #[allow(dead_code)]
    Stream(()),
}

impl From<identify::Event> for PortKeyBehaviourEvent {
    fn from(event: identify::Event) -> Self {
        PortKeyBehaviourEvent::Identify(event)
    }
}

impl From<ping::Event> for PortKeyBehaviourEvent {
    fn from(event: ping::Event) -> Self {
        PortKeyBehaviourEvent::Ping(event)
    }
}

impl From<mdns::Event> for PortKeyBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        PortKeyBehaviourEvent::Mdns(event)
    }
}

impl From<relay::client::Event> for PortKeyBehaviourEvent {
    fn from(event: relay::client::Event) -> Self {
        PortKeyBehaviourEvent::Relay(event)
    }
}

impl From<dcutr::Event> for PortKeyBehaviourEvent {
    fn from(event: dcutr::Event) -> Self {
        PortKeyBehaviourEvent::Dcutr(event)
    }
}

impl From<()> for PortKeyBehaviourEvent {
    fn from(_: ()) -> Self {
        PortKeyBehaviourEvent::Stream(())
    }
}
