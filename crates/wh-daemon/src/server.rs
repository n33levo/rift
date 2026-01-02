//! Daemon Server
//!
//! Main daemon that orchestrates the P2P network, sessions, and UI updates.

use futures::StreamExt;
use wh_core::{
    open_tunnel_stream,
    send_secrets, receive_secrets,
    NetworkEvent, PeerNetwork, RiftConfig, Result, PeerId,
    secrets::EnvVault,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
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

    /// Incoming connection request (waiting for approval)
    IncomingConnectionRequest { peer_id: String },

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
        auto_approve: bool,
    },

    /// Connect to a peer
    Connect {
        link: String,
        port: u16,
        local_port: Option<u16>,
        bind_addr: String,
    },

    /// Approve an incoming connection
    ApproveConnection { peer_id: String },

    /// Deny an incoming connection
    DenyConnection { peer_id: String },

    /// Stop a session
    StopSession { session_id: u64 },

    /// Shutdown daemon
    Shutdown,
}

/// Shared traffic stats (atomic for cross-task updates)
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc as StdArc;

#[derive(Debug, Default)]
pub struct TrafficStats {
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,
    pub active_connections: AtomicU64,
}

/// Main daemon server
pub struct DaemonServer {
    /// Configuration
    #[allow(dead_code)]
    config: RiftConfig,

    /// P2P Network (owned, not shared)
    network: Option<PeerNetwork>,

    /// Peer ID (cached after start)
    peer_id: String,

    /// Rift link (cached after start)
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

    /// Pending connection approvals (peer_id -> response channel)
    pending_approvals: HashMap<String, oneshot::Sender<bool>>,
    
    /// Traffic statistics (shared with spawned tasks)
    traffic_stats: StdArc<TrafficStats>,
}

impl DaemonServer {
    /// Create a new daemon server
    pub async fn new(config: RiftConfig) -> Result<Self> {
        let network = PeerNetwork::new(config.clone()).await?;
        let peer_id = network.peer_id().to_string();
        let link = network.rift_link();

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
            pending_approvals: HashMap::new(),
            traffic_stats: StdArc::new(TrafficStats::default()),
        })
    }

    /// Get the command sender
    pub fn command_sender(&self) -> mpsc::Sender<DaemonCommand> {
        self.command_tx.clone()
    }

    /// Take the event receiver
    pub fn take_event_receiver(&mut self) -> mpsc::Receiver<DaemonEvent> {
        let (_new_tx, new_rx) = mpsc::channel(256);
        let old_rx = std::mem::replace(&mut self.event_rx, new_rx);
        // DON'T replace event_tx - keep sending to the channel we're giving out
        old_rx
    }

    /// Get peer ID
    pub async fn peer_id(&self) -> String {
        self.peer_id.clone()
    }

    /// Get Rift link
    pub async fn rift_link(&self) -> String {
        self.link.clone()
    }

    /// Start the daemon
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting Rift daemon...");

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
        let traffic_stats = self.traffic_stats.clone();

        // Track share session target port
        let mut share_port: Option<u16> = None;
        
        // Track auto-approve setting
        let mut auto_approve = false;
        
        // Track secrets to share
        let mut share_secrets: Option<EnvVault> = None;
        
        // Track connect session info
        let mut connect_info: Option<(PeerId, u16, TcpListener)> = None;
        let stream_control = network.stream_control();
        
        // Stats update timer - send stats every 100ms for smooth graph updates
        let mut stats_interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        stats_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Main event loop
        while self.running {
            tokio::select! {
                // Periodic stats update
                _ = stats_interval.tick() => {
                    let _ = event_tx.send(DaemonEvent::StatsUpdate {
                        bytes_sent: traffic_stats.bytes_sent.load(Ordering::Relaxed),
                        bytes_received: traffic_stats.bytes_received.load(Ordering::Relaxed),
                        active_connections: traffic_stats.active_connections.load(Ordering::Relaxed),
                    }).await;
                }
                // Handle commands
                Some(command) = self.command_rx.recv() => {
                    match command {
                        DaemonCommand::Share { port, secrets_path, auto_approve: auto_approve_flag } => {
                            info!("Share command received for port {} (auto_approve={})", port, auto_approve_flag);
                            share_port = Some(port);
                            auto_approve = auto_approve_flag;
                            
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
                        DaemonCommand::Connect { link, port, local_port, bind_addr } => {
                            info!("Connect command received for {} port {}", link, port);
                            
                            // Retry connection with backoff for peer discovery
                            // Give mDNS time to discover the peer (usually takes 100-200ms)
                            let mut retry_count = 0;
                            let max_retries = 20;
                            let retry_delay = tokio::time::Duration::from_millis(250);
                            
                            let connection_result = loop {
                                // Poll network to process mDNS events
                                let _ = network.poll_once().await;
                                
                                match network.connect(&link).await {
                                    Ok(peer_id) => break Ok(peer_id),
                                    Err(e) if retry_count < max_retries => {
                                        if retry_count == 0 {
                                            info!("Waiting for peer discovery...");
                                        }
                                        retry_count += 1;
                                        tokio::time::sleep(retry_delay).await;
                                    }
                                    Err(e) => break Err(e),
                                }
                            };
                            
                            match connection_result {
                                Ok(peer_id) => {
                                    info!("Connected to peer {}", peer_id);
                                    // Start local TCP listener
                                    let local = local_port.unwrap_or(port);
                                    match TcpListener::bind(format!("{}:{}", bind_addr, local)).await {
                                        Ok(listener) => {
                                            info!("Local proxy listening on {}:{}", bind_addr, local);
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
                        DaemonCommand::ApproveConnection { peer_id } => {
                            info!("Approving connection from {}", peer_id);
                            if let Some(tx) = self.pending_approvals.remove(&peer_id) {
                                let _ = tx.send(true);
                            }
                        }
                        DaemonCommand::DenyConnection { peer_id } => {
                            info!("Denying connection from {}", peer_id);
                            if let Some(tx) = self.pending_approvals.remove(&peer_id) {
                                let _ = tx.send(false);
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
                        let peer_id_str = peer_id.to_string();
                        info!("Incoming stream from {} - checking approval...", peer_id_str);
                        
                        // Check if auto-approve is enabled
                        let approved = if auto_approve {
                            info!("Auto-approving connection from {}", peer_id_str);
                            true
                        } else {
                            // Request approval from UI
                            let (approval_tx, approval_rx) = oneshot::channel();
                            self.pending_approvals.insert(peer_id_str.clone(), approval_tx);
                            
                            let _ = event_tx.send(DaemonEvent::IncomingConnectionRequest {
                                peer_id: peer_id_str.clone(),
                            }).await;
                            
                            // Wait for approval (with timeout)
                            match tokio::time::timeout(
                                std::time::Duration::from_secs(30),
                                approval_rx
                            ).await {
                                Ok(Ok(approved)) => approved,
                                Ok(Err(_)) => {
                                    warn!("Approval channel closed for {}", peer_id_str);
                                    false
                                }
                                Err(_) => {
                                    warn!("Approval timeout for {}", peer_id_str);
                                    self.pending_approvals.remove(&peer_id_str);
                                    false
                                }
                            }
                        };
                        
                        if approved {
                            info!("Connection approved - bridging to localhost:{}", port);
                            let stats = traffic_stats.clone();
                            // Spawn a task to bridge this stream to localhost:port with traffic tracking
                            tokio::spawn(async move {
                                stats.active_connections.fetch_add(1, Ordering::Relaxed);
                                match bridge_with_stats(stream, port, stats.clone()).await {
                                    Ok((sent, recv)) => {
                                        debug!("Stream from {} closed. Sent: {}, Recv: {}", peer_id, sent, recv);
                                    }
                                    Err(e) => {
                                        warn!("Stream bridge ended: {}", e);
                                    }
                                }
                                stats.active_connections.fetch_sub(1, Ordering::Relaxed);
                            });
                        } else {
                            info!("Connection denied from {}", peer_id_str);
                            drop(stream);
                        }
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
                            let request: wh_core::secrets::SecretsRequest = match receive_secrets(&mut read).await {
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
                            let stats = traffic_stats.clone();
                            
                            tokio::spawn(async move {
                                stats.active_connections.fetch_add(1, Ordering::Relaxed);
                                match open_tunnel_stream(&mut control, peer_id).await {
                                    Ok(stream) => {
                                        // Convert futures AsyncRead/Write to tokio
                                        let stream = stream.compat();
                                        let (mut stream_read, mut stream_write) = tokio::io::split(stream);
                                        let (mut tcp_read, mut tcp_write) = tcp_stream.into_split();
                                        
                                        // Bidirectional copy with stats tracking
                                        let stats_clone = stats.clone();
                                        tokio::select! {
                                            _r = async {
                                                let mut buf = [0u8; 8192];
                                                let mut total = 0u64;
                                                loop {
                                                    match tokio::io::AsyncReadExt::read(&mut stream_read, &mut buf).await {
                                                        Ok(0) => break,
                                                        Ok(n) => {
                                                            if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut tcp_write, &buf[..n]).await {
                                                                debug!("Stream->TCP write error: {}", e);
                                                                break;
                                                            }
                                                            total += n as u64;
                                                            stats_clone.bytes_received.fetch_add(n as u64, Ordering::Relaxed);
                                                        }
                                                        Err(e) => {
                                                            debug!("Stream->TCP read error: {}", e);
                                                            break;
                                                        }
                                                    }
                                                }
                                                total
                                            } => {}
                                            _r = async {
                                                let mut buf = [0u8; 8192];
                                                let mut total = 0u64;
                                                loop {
                                                    match tokio::io::AsyncReadExt::read(&mut tcp_read, &mut buf).await {
                                                        Ok(0) => break,
                                                        Ok(n) => {
                                                            if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut stream_write, &buf[..n]).await {
                                                                debug!("TCP->Stream write error: {}", e);
                                                                break;
                                                            }
                                                            total += n as u64;
                                                            stats.bytes_sent.fetch_add(n as u64, Ordering::Relaxed);
                                                        }
                                                        Err(e) => {
                                                            debug!("TCP->Stream read error: {}", e);
                                                            break;
                                                        }
                                                    }
                                                }
                                                total
                                            } => {}
                                        }
                                        debug!("Tunnel connection to {} closed", peer_id);
                                    }
                                    Err(e) => {
                                        error!("Failed to open stream to peer: {}", e);
                                    }
                                }
                                stats.active_connections.fetch_sub(1, Ordering::Relaxed);
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

/// Bridge a stream to a local TCP port with traffic stats tracking
async fn bridge_with_stats(
    stream: libp2p::Stream,
    target_port: u16,
    stats: StdArc<TrafficStats>,
) -> wh_core::Result<(u64, u64)> {
    use tokio::net::TcpStream;
    use wh_core::RiftError;
    
    let tcp = TcpStream::connect(format!("127.0.0.1:{}", target_port))
        .await
        .map_err(|e| RiftError::ProxyError(format!("Failed to connect to local port {}: {}", target_port, e)))?;

    // Convert futures AsyncRead/Write to tokio AsyncRead/Write using compat
    let stream = stream.compat();
    
    let (mut tcp_read, mut tcp_write) = tcp.into_split();
    let (mut stream_read, mut stream_write) = tokio::io::split(stream);

    let stats_send = stats.clone();
    let stats_recv = stats.clone();

    // Bidirectional copy with stats tracking
    let send_task = async move {
        let mut buf = [0u8; 8192];
        let mut total = 0u64;
        loop {
            match tokio::io::AsyncReadExt::read(&mut tcp_read, &mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut stream_write, &buf[..n]).await {
                        debug!("TCP->Stream write error: {}", e);
                        break;
                    }
                    total += n as u64;
                    stats_send.bytes_sent.fetch_add(n as u64, Ordering::Relaxed);
                }
                Err(e) => {
                    debug!("TCP->Stream read error: {}", e);
                    break;
                }
            }
        }
        total
    };

    let recv_task = async move {
        let mut buf = [0u8; 8192];
        let mut total = 0u64;
        loop {
            match tokio::io::AsyncReadExt::read(&mut stream_read, &mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut tcp_write, &buf[..n]).await {
                        debug!("Stream->TCP write error: {}", e);
                        break;
                    }
                    total += n as u64;
                    stats_recv.bytes_received.fetch_add(n as u64, Ordering::Relaxed);
                }
                Err(e) => {
                    debug!("Stream->TCP read error: {}", e);
                    break;
                }
            }
        }
        total
    };

    let (sent, recv) = tokio::join!(send_task, recv_task);
    Ok((sent, recv))
}
