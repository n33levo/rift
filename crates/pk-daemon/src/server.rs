//! Daemon Server
//!
//! Main daemon that orchestrates the P2P network, sessions, and UI updates.

use futures::StreamExt;
use pk_core::{
    bridge_stream_to_tcp, open_tunnel_stream,
    send_secrets, receive_secrets,
    NetworkEvent, PeerNetwork, PortKeyConfig, Result, PeerId,
    secrets::EnvVault,
};
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tracing::{debug, error, info, warn};

/// Events from the daemon to the UI
#[derive(Debug, Clone)]
pub enum DaemonEvent {
    /// Network is ready
    Ready { peer_id: String, link: String },

    /// Listening on address
    Listening { address: String },

    /// Peer connected
    PeerConnected { peer_id: String },

    /// Peer disconnected
    PeerDisconnected { peer_id: String },

    /// Tunnel established
    TunnelEstablished { peer_id: String, port: u16 },

    /// New tunnel connection
    TunnelConnection { connection_id: u64 },

    /// Secrets received
    SecretsReceived { count: usize },

    /// Statistics update
    StatsUpdate {
        bytes_sent: u64,
        bytes_received: u64,
        active_connections: u64,
    },

    /// Error occurred
    Error { message: String },

    /// Shutdown
    Shutdown,
}

/// Commands to the daemon
#[derive(Debug)]
pub enum DaemonCommand {
    /// Share a port
    Share {
        port: u16,
        secrets_path: Option<PathBuf>,
    },

    /// Connect to a peer
    Connect {
        link: String,
        port: u16,
        local_port: Option<u16>,
    },

    /// Stop a session
    StopSession { session_id: u64 },

    /// Shutdown daemon
    Shutdown,
}

/// Main daemon server
pub struct DaemonServer {
    /// Configuration
    #[allow(dead_code)]
    config: PortKeyConfig,

    /// P2P Network (owned, not shared)
    network: Option<PeerNetwork>,

    /// Peer ID (cached after start)
    peer_id: String,

    /// PortKey link (cached after start)
    link: String,

    /// Event sender
    event_tx: mpsc::Sender<DaemonEvent>,

    /// Event receiver
    event_rx: mpsc::Receiver<DaemonEvent>,

    /// Command sender
    command_tx: mpsc::Sender<DaemonCommand>,

    /// Command receiver
    command_rx: mpsc::Receiver<DaemonCommand>,

    /// Running flag
    running: bool,
}

impl DaemonServer {
    /// Create a new daemon server
    pub async fn new(config: PortKeyConfig) -> Result<Self> {
        let network = PeerNetwork::new(config.clone()).await?;
        let peer_id = network.peer_id().to_string();
        let link = network.portkey_link();

        let (event_tx, event_rx) = mpsc::channel(256);
        let (command_tx, command_rx) = mpsc::channel(64);

        Ok(Self {
            config,
            network: Some(network),
            peer_id,
            link,
            event_tx,
            event_rx,
            command_tx,
            command_rx,
            running: false,
        })
    }

    /// Get the command sender
    pub fn command_sender(&self) -> mpsc::Sender<DaemonCommand> {
        self.command_tx.clone()
    }

    /// Take the event receiver
    pub fn take_event_receiver(&mut self) -> mpsc::Receiver<DaemonEvent> {
        let (new_tx, new_rx) = mpsc::channel(256);
        let old_rx = std::mem::replace(&mut self.event_rx, new_rx);
        self.event_tx = new_tx;
        old_rx
    }

    /// Get peer ID
    pub async fn peer_id(&self) -> String {
        self.peer_id.clone()
    }

    /// Get PortKey link
    pub async fn portkey_link(&self) -> String {
        self.link.clone()
    }

    /// Start the daemon
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting PortKey daemon...");

        // Start listening
        if let Some(ref mut network) = self.network {
            let addresses = network.start_listening().await?;

            for addr in &addresses {
                let _ = self
                    .event_tx
                    .send(DaemonEvent::Listening {
                        address: addr.to_string(),
                    })
                    .await;
            }
        }

        // Send ready event
        let _ = self
            .event_tx
            .send(DaemonEvent::Ready {
                peer_id: self.peer_id.clone(),
                link: self.link.clone(),
            })
            .await;

        self.running = true;
        Ok(())
    }

    /// Run the daemon main loop
    pub async fn run(&mut self) -> Result<()> {
        // Take ownership of network for the run loop
        let mut network = self.network.take().expect("Network should be available");
        let mut network_rx = network.take_event_receiver();
        let mut incoming_streams = network.take_incoming_streams();
        let mut incoming_secrets_streams = network.take_incoming_secrets_streams();
        let event_tx = self.event_tx.clone();

        // Track share session target port
        let mut share_port: Option<u16> = None;
        
        // Track secrets to share
        let mut share_secrets: Option<EnvVault> = None;
        
        // Track connect session info
        let mut connect_info: Option<(PeerId, u16, TcpListener)> = None;
        let stream_control = network.stream_control();

        // Main event loop
        while self.running {
            tokio::select! {
                // Handle commands
                Some(command) = self.command_rx.recv() => {
                    match command {
                        DaemonCommand::Share { port, secrets_path } => {
                            info!("Share command received for port {}", port);
                            share_port = Some(port);
                            
                            // Load secrets if provided
                            if let Some(path) = secrets_path {
                                match EnvVault::from_file(&path) {
                                    Ok(vault) => {
                                        info!("Loaded secrets from {}", path.display());
                                        share_secrets = Some(vault);
                                    }
                                    Err(e) => {
                                        error!("Failed to load secrets from {}: {}", path.display(), e);
                                        let _ = event_tx.send(DaemonEvent::Error {
                                            message: format!("Failed to load secrets: {}", e),
                                        }).await;
                                    }
                                }
                            }
                        }
                        DaemonCommand::Connect { link, port, local_port } => {
                            info!("Connect command received for {} port {}", link, port);
                            match network.connect(&link).await {
                                Ok(peer_id) => {
                                    info!("Connected to peer {}", peer_id);
                                    // Start local TCP listener
                                    let local = local_port.unwrap_or(port);
                                    match TcpListener::bind(format!("127.0.0.1:{}", local)).await {
                                        Ok(listener) => {
                                            info!("Local proxy listening on 127.0.0.1:{}", local);
                                            connect_info = Some((peer_id, port, listener));
                                            let _ = event_tx.send(DaemonEvent::TunnelEstablished {
                                                peer_id: peer_id.to_string(),
                                                port: local,
                                            }).await;
                                        }
                                        Err(e) => {
                                            error!("Failed to bind local port {}: {}", local, e);
                                            let _ = event_tx.send(DaemonEvent::Error {
                                                message: format!("Failed to bind port {}: {}", local, e),
                                            }).await;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to connect to {}: {}", link, e);
                                    let _ = event_tx.send(DaemonEvent::Error {
                                        message: format!("Failed to connect: {}", e),
                                    }).await;
                                }
                            }
                        }
                        DaemonCommand::Shutdown => {
                            info!("Shutdown command received");
                            self.running = false;
                        }
                        DaemonCommand::StopSession { .. } => {}
                    }
                }

                // Handle incoming streams (host side - share)
                Some((peer_id, stream)) = incoming_streams.next() => {
                    if let Some(port) = share_port {
                        info!("Incoming stream from {} - bridging to localhost:{}", peer_id, port);
                        // Spawn a task to bridge this stream to localhost:port
                        tokio::spawn(async move {
                            if let Err(e) = bridge_stream_to_tcp(stream, port).await {
                                warn!("Stream bridge ended: {}", e);
                            }
                            debug!("Stream from {} closed", peer_id);
                        });
                    } else {
                        warn!("Received stream but no share session active");
                    }
                }

                // Handle incoming secrets requests (host side - share with secrets)
                Some((peer_id, stream)) = incoming_secrets_streams.next() => {
                    if let Some(ref vault) = share_secrets {
                        info!("Incoming secrets request from {}", peer_id);
                        let vault = vault.clone();
                        
                        tokio::spawn(async move {
                            let stream = stream.compat();
                            let (mut read, mut write) = tokio::io::split(stream);
                            
                            // 1. Read SecretsRequest
                            let request: pk_core::secrets::SecretsRequest = match receive_secrets(&mut read).await {
                                Ok(req) => req,
                                Err(e) => {
                                    error!("Failed to receive request: {}", e);
                                    return;
                                }
                            };
                            
                            // 2. Encrypt secrets for the requester's public key
                            let response = match vault.encrypt_for_peer(&request.public_key) {
                                Ok(resp) => resp,
                                Err(e) => {
                                    error!("Failed to encrypt secrets: {}", e);
                                    return;
                                }
                            };
                            
                            // 3. Send SecretsResponse
                            if let Err(e) = send_secrets(&mut write, &response).await {
                                error!("Failed to send response: {}", e);
                                return;
                            }
                            
                            info!("Secrets sent to {}", peer_id);
                        });
                    } else {
                        warn!("Received secrets request but no secrets configured");
                    }
                }

                // Handle incoming TCP connections (client side - connect)  
                result = async {
                    if let Some((_, _, ref listener)) = connect_info {
                        listener.accept().await
                    } else {
                        // No listener, pend forever
                        std::future::pending().await
                    }
                } => {
                    if let Ok((tcp_stream, addr)) = result {
                        if let Some((peer_id, _remote_port, _)) = &connect_info {
                            info!("Incoming TCP connection from {} - opening stream to peer", addr);
                            let peer_id = *peer_id;
                            let mut control = stream_control.clone();
                            
                            tokio::spawn(async move {
                                match open_tunnel_stream(&mut control, peer_id).await {
                                    Ok(stream) => {
                                        // Convert futures AsyncRead/Write to tokio
                                        let stream = stream.compat();
                                        let (mut stream_read, mut stream_write) = tokio::io::split(stream);
                                        let (mut tcp_read, mut tcp_write) = tcp_stream.into_split();
                                        
                                        // Bidirectional copy
                                        tokio::select! {
                                            r = tokio::io::copy(&mut stream_read, &mut tcp_write) => {
                                                if let Err(e) = r {
                                                    debug!("Stream->TCP ended: {}", e);
                                                }
                                            }
                                            r = tokio::io::copy(&mut tcp_read, &mut stream_write) => {
                                                if let Err(e) = r {
                                                    debug!("TCP->Stream ended: {}", e);
                                                }
                                            }
                                        }
                                        debug!("Tunnel connection to {} closed", peer_id);
                                    }
                                    Err(e) => {
                                        error!("Failed to open stream to peer: {}", e);
                                    }
                                }
                            });
                        }
                    }
                }

                // Handle network events
                Some(event) = network_rx.recv() => {
                    Self::handle_network_event(&event_tx, event).await;
                }

                // Poll the swarm to drive progress
                _ = network.poll_once() => {}
            }
        }

        // Cleanup
        network.shutdown().await;
        let _ = self.event_tx.send(DaemonEvent::Shutdown).await;
        
        Ok(())
    }

    /// Handle a network event (static method for use in run loop)
    async fn handle_network_event(event_tx: &mpsc::Sender<DaemonEvent>, event: NetworkEvent) {
        match event {
            NetworkEvent::Listening { address } => {
                let _ = event_tx
                    .send(DaemonEvent::Listening {
                        address: address.to_string(),
                    })
                    .await;
            }
            NetworkEvent::PeerConnected { peer_id } => {
                info!("Peer connected: {}", peer_id);
                let _ = event_tx
                    .send(DaemonEvent::PeerConnected {
                        peer_id: peer_id.to_string(),
                    })
                    .await;
            }
            NetworkEvent::PeerDisconnected { peer_id } => {
                info!("Peer disconnected: {}", peer_id);
                let _ = event_tx
                    .send(DaemonEvent::PeerDisconnected {
                        peer_id: peer_id.to_string(),
                    })
                    .await;
            }
            NetworkEvent::HolePunchSucceeded { peer_id } => {
                info!("Hole punch succeeded with {}", peer_id);
            }
            NetworkEvent::Error { message } => {
                error!("Network error: {}", message);
                let _ = event_tx.send(DaemonEvent::Error { message }).await;
            }
        }
    }
}
