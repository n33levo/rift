//! Swarm Management for Rift
//!
//! High-level interface for managing the libp2p swarm and raw stream tunneling.

use futures::StreamExt;
use libp2p::{
    identify, mdns, ping,
    swarm::SwarmEvent,
    Multiaddr, PeerId, Swarm, Stream,
};
use libp2p_stream as stream;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tracing::{debug, error, info, warn};

use super::behaviour::{RiftBehaviour, RiftBehaviourEvent, TUNNEL_PROTOCOL, SECRETS_PROTOCOL};
use super::identity::PeerIdentity;
use crate::config::RiftConfig;
use crate::error::{RiftError, Result};

/// Events emitted by the peer network
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// Listening on an address
    Listening { address: Multiaddr },
    /// Peer connected
    PeerConnected { peer_id: PeerId },
    /// Peer disconnected  
    PeerDisconnected { peer_id: PeerId },
    /// Hole punch succeeded
    HolePunchSucceeded { peer_id: PeerId },
    /// Error occurred
    Error { message: String },
}

/// Connection information for a peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub addresses: Vec<Multiaddr>,
    pub connected_at: Instant,
}

/// High-level peer network manager
pub struct PeerNetwork {
    /// Our identity
    identity: PeerIdentity,
    /// Configuration
    config: RiftConfig,
    /// The libp2p swarm
    swarm: Swarm<RiftBehaviour>,
    /// Known peers
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
    /// Event sender
    event_tx: mpsc::Sender<NetworkEvent>,
    /// Event receiver
    event_rx: Option<mpsc::Receiver<NetworkEvent>>,
    /// Running flag
    running: bool,
}

impl PeerNetwork {
    /// Create a new peer network
    pub async fn new(config: RiftConfig) -> Result<Self> {
        let identity = PeerIdentity::load_or_generate(&config.identity_path)?;
        let local_peer_id = *identity.peer_id();
        let keypair = identity.keypair().clone();

        info!("Local peer ID: {}", local_peer_id);

        // Build the swarm
        let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_quic()
            .with_relay_client(libp2p::noise::Config::new, libp2p::yamux::Config::default)
            .map_err(|e| RiftError::NetworkInitialization(e.to_string()))?
            .with_behaviour(|key, relay| {
                let identify = identify::Behaviour::new(
                    identify::Config::new("/rift/id/1.0.0".to_string(), key.public())
                        .with_agent_version(format!("rift/{}", env!("CARGO_PKG_VERSION")))
                        .with_push_listen_addr_updates(true),
                );

                let ping = ping::Behaviour::new(
                    ping::Config::new()
                        .with_interval(std::time::Duration::from_secs(15))
                        .with_timeout(std::time::Duration::from_secs(10)),
                );

                let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;
                let dcutr = libp2p::dcutr::Behaviour::new(local_peer_id);
                let stream = stream::Behaviour::new();

                Ok(RiftBehaviour {
                    identify,
                    ping,
                    mdns,
                    relay,
                    dcutr,
                    stream,
                })
            })
            .map_err(|e| RiftError::NetworkInitialization(e.to_string()))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
            .build();

        let (event_tx, event_rx) = mpsc::channel(256);

        let mut network = Self {
            identity,
            config: config.clone(),
            swarm,
            peers: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Some(event_rx),
            running: false,
        };

        // Dial bootstrap peers for relay/DHT connectivity
        for peer_addr in &config.bootstrap_peers {
            if let Ok(addr) = peer_addr.parse::<Multiaddr>() {
                info!("Dialing bootstrap peer: {}", addr);
                let _ = network.swarm.dial(addr);
            }
        }

        Ok(network)
    }

    /// Get our peer ID
    pub fn peer_id(&self) -> &PeerId {
        self.identity.peer_id()
    }

    /// Get our Rift link
    pub fn rift_link(&self) -> String {
        self.identity.to_rift_link()
    }

    /// Take the event receiver
    pub fn take_event_receiver(&mut self) -> mpsc::Receiver<NetworkEvent> {
        self.event_rx.take().expect("Event receiver already taken")
    }

    /// Take incoming streams receiver for handling tunnel connections
    pub fn take_incoming_streams(&mut self) -> stream::IncomingStreams {
        // Clone the control and create new incoming streams
        self.swarm
            .behaviour()
            .stream
            .new_control()
            .accept(TUNNEL_PROTOCOL)
            .unwrap()
    }
    
    /// Take incoming secrets streams receiver
    pub fn take_incoming_secrets_streams(&mut self) -> stream::IncomingStreams {
        self.swarm
            .behaviour()
            .stream
            .new_control()
            .accept(SECRETS_PROTOCOL)
            .unwrap()
    }

    /// Get a control handle for opening outgoing streams
    pub fn stream_control(&self) -> stream::Control {
        self.swarm.behaviour().stream.new_control()
    }

    /// Start listening
    pub async fn start_listening(&mut self) -> Result<Vec<Multiaddr>> {
        let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/udp/{}/quic-v1", self.config.listen_port)
            .parse()
            .map_err(|e| RiftError::NetworkInitialization(format!("Invalid address: {}", e)))?;

        self.swarm
            .listen_on(listen_addr)
            .map_err(|e| RiftError::NetworkInitialization(e.to_string()))?;

        // Also try IPv6
        if let Ok(addr) = format!("/ip6/::/udp/{}/quic-v1", self.config.listen_port).parse() {
            let _ = self.swarm.listen_on(addr);
        }

        self.running = true;
        Ok(self.swarm.listeners().cloned().collect())
    }

    /// Connect to a peer by their Rift link
    pub async fn connect(&mut self, link: &str) -> Result<PeerId> {
        let peer_id = PeerIdentity::parse_rift_link(link)?;
        self.dial_peer(peer_id).await?;
        Ok(peer_id)
    }

    /// Dial a specific peer
    pub async fn dial_peer(&mut self, peer_id: PeerId) -> Result<()> {
        self.swarm
            .dial(peer_id)
            .map_err(|e| RiftError::DialError(e.to_string()))?;
        Ok(())
    }

    /// Add a peer address
    pub fn add_peer_address(&mut self, peer_id: PeerId, addr: Multiaddr) {
        self.swarm.add_peer_address(peer_id, addr);
    }

    /// Run the network event loop - call this in a spawned task
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting Rift network...");

        while self.running {
            if let Some(event) = self.swarm.next().await {
                if let Err(e) = self.handle_swarm_event(event).await {
                    error!("Error handling swarm event: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Poll swarm once - for manual event loop control
    pub async fn poll_once(&mut self) -> Option<()> {
        tokio::select! {
            event = self.swarm.select_next_some() => {
                let _ = self.handle_swarm_event(event).await;
                Some(())
            }
        }
    }

    async fn handle_swarm_event(&mut self, event: SwarmEvent<RiftBehaviourEvent>) -> Result<()> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {}", address);
                let _ = self.event_tx.send(NetworkEvent::Listening { address }).await;
            }

            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connected to peer: {}", peer_id);
                let info = PeerInfo {
                    peer_id,
                    addresses: Vec::new(),
                    connected_at: Instant::now(),
                };
                self.peers.write().await.insert(peer_id, info);
                let _ = self.event_tx.send(NetworkEvent::PeerConnected { peer_id }).await;
            }

            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("Disconnected from peer: {}", peer_id);
                self.peers.write().await.remove(&peer_id);
                let _ = self.event_tx.send(NetworkEvent::PeerDisconnected { peer_id }).await;
            }

            SwarmEvent::Behaviour(event) => {
                self.handle_behaviour_event(event).await?;
            }

            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    warn!("Failed to connect to {}: {}", peer_id, error);
                }
            }

            _ => {}
        }

        Ok(())
    }

    async fn handle_behaviour_event(&mut self, event: RiftBehaviourEvent) -> Result<()> {
        match event {
            RiftBehaviourEvent::Mdns(mdns::Event::Discovered(peers)) => {
                for (peer_id, addr) in peers {
                    debug!("Discovered peer via mDNS: {} at {}", peer_id, addr);
                    self.swarm.add_peer_address(peer_id, addr);
                }
            }

            RiftBehaviourEvent::Identify(identify::Event::Received { peer_id, info, .. }) => {
                debug!("Identified peer {}: {:?}", peer_id, info.agent_version);
                for addr in &info.listen_addrs {
                    self.swarm.add_peer_address(peer_id, addr.clone());
                }
                if let Some(peer_info) = self.peers.write().await.get_mut(&peer_id) {
                    peer_info.addresses = info.listen_addrs;
                }
            }

            RiftBehaviourEvent::Dcutr(libp2p::dcutr::Event { remote_peer_id, result }) => {
                match result {
                    Ok(_) => {
                        info!("Hole punch succeeded with {}", remote_peer_id);
                        let _ = self.event_tx.send(NetworkEvent::HolePunchSucceeded { peer_id: remote_peer_id }).await;
                    }
                    Err(e) => {
                        warn!("Hole punch failed with {}: {:?}", remote_peer_id, e);
                    }
                }
            }

            // Stream events are handled separately via incoming_streams
            RiftBehaviourEvent::Stream(_) => {}

            _ => {}
        }

        Ok(())
    }

    /// Shutdown the network
    pub async fn shutdown(&mut self) {
        info!("Shutting down Rift network...");
        self.running = false;
    }
}

/// Open a new outgoing stream to a peer for tunneling
pub async fn open_tunnel_stream(
    control: &mut stream::Control,
    peer_id: PeerId,
) -> Result<Stream> {
    control
        .open_stream(peer_id, TUNNEL_PROTOCOL)
        .await
        .map_err(|e| RiftError::StreamError(format!("Failed to open stream: {:?}", e)))
}

/// Bridge a QUIC stream to a local TCP connection
/// This is the core tunnel logic - just pump bytes bidirectionally
pub async fn bridge_stream_to_tcp(stream: Stream, target_port: u16) -> Result<()> {
    let tcp = TcpStream::connect(format!("127.0.0.1:{}", target_port))
        .await
        .map_err(|e| RiftError::ProxyError(format!("Failed to connect to local port {}: {}", target_port, e)))?;

    // Convert futures AsyncRead/Write to tokio AsyncRead/Write using compat
    let stream = stream.compat();
    
    // Use copy_bidirectional for efficient byte pumping
    let (mut tcp_read, mut tcp_write) = tcp.into_split();
    let (mut stream_read, mut stream_write) = tokio::io::split(stream);

    // Bidirectional copy
    let client_to_server = tokio::io::copy(&mut stream_read, &mut tcp_write);
    let server_to_client = tokio::io::copy(&mut tcp_read, &mut stream_write);

    tokio::select! {
        result = client_to_server => {
            if let Err(e) = result {
                debug!("Stream->TCP copy ended: {}", e);
            }
        }
        result = server_to_client => {
            if let Err(e) = result {
                debug!("TCP->Stream copy ended: {}", e);
            }
        }
    }

    Ok(())
}

/// Send a serializable message with length prefix
pub async fn send_secrets<T: serde::Serialize, W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    data: &T,
) -> Result<()> {
    use tokio::io::AsyncWriteExt;
    
    let bytes = bincode::serialize(data)?;
    let len = bytes.len() as u32;
    
    writer.write_all(&len.to_be_bytes()).await
        .map_err(|e| RiftError::StreamError(format!("Failed to write length: {}", e)))?;
    writer.write_all(&bytes).await
        .map_err(|e| RiftError::StreamError(format!("Failed to write data: {}", e)))?;
    
    Ok(())
}

/// Receive a deserializable message with length prefix
pub async fn receive_secrets<T: serde::de::DeserializeOwned, R: tokio::io::AsyncRead + Unpin>(
    reader: &mut R,
) -> Result<T> {
    use tokio::io::AsyncReadExt;
    
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await
        .map_err(|e| RiftError::StreamError(format!("Failed to read length: {}", e)))?;
    
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > 10 * 1024 * 1024 {
        return Err(RiftError::StreamError("Message too large".to_string()));
    }
    
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await
        .map_err(|e| RiftError::StreamError(format!("Failed to read data: {}", e)))?;
    
    bincode::deserialize(&buf)
        .map_err(|e| RiftError::Serialization(format!("Failed to deserialize: {}", e)))
}

/// Send secrets to a peer over a dedicated stream
pub async fn send_secrets_to_peer(
    control: &mut stream::Control,
    peer_id: PeerId,
    secrets_response: &crate::secrets::SecretsResponse,
) -> Result<()> {
    use super::behaviour::SECRETS_PROTOCOL;
    use tokio::io::AsyncWriteExt;
    
    // Open a stream for secrets
    let stream = control
        .open_stream(peer_id, SECRETS_PROTOCOL)
        .await
        .map_err(|e| RiftError::StreamError(format!("Failed to open secrets stream: {:?}", e)))?;
    
    // Serialize and send
    let data = bincode::serialize(secrets_response)
        .map_err(|e| RiftError::Serialization(format!("Failed to serialize secrets: {}", e)))?;
    
    let mut stream = stream.compat();
    stream.write_all(&data).await
        .map_err(|e| RiftError::StreamError(format!("Failed to send secrets: {}", e)))?;
    stream.shutdown().await
        .map_err(|e| RiftError::StreamError(format!("Failed to close secrets stream: {}", e)))?;
    
    info!("Sent {} bytes of encrypted secrets to {}", data.len(), peer_id);
    Ok(())
}

/// Receive secrets from a stream
pub async fn receive_secrets_from_stream(stream: Stream) -> Result<crate::secrets::SecretsResponse> {
    use tokio::io::AsyncReadExt;
    
    let mut stream = stream.compat();
    let mut data = Vec::new();
    
    stream.read_to_end(&mut data).await
        .map_err(|e| RiftError::StreamError(format!("Failed to read secrets: {}", e)))?;
    
    let response: crate::secrets::SecretsResponse = bincode::deserialize(&data)
        .map_err(|e| RiftError::Serialization(format!("Failed to deserialize secrets: {}", e)))?;
    
    info!("Received {} bytes of encrypted secrets", data.len());
    Ok(response)
}

