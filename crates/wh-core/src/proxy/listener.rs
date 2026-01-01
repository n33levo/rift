//! TCP Listener for Proxy
//!
//! Accepts incoming TCP connections and forwards them through the tunnel.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::error::{RiftError, Result};

/// Statistics for the proxy
#[derive(Debug, Default)]
pub struct ProxyStats {
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,
    pub connections_total: AtomicU64,
    pub connections_active: AtomicU64,
}

impl ProxyStats {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn record_sent(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn connection_opened(&self) {
        self.connections_total.fetch_add(1, Ordering::Relaxed);
        self.connections_active.fetch_add(1, Ordering::Relaxed);
    }

    pub fn connection_closed(&self) {
        self.connections_active.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn get_bytes_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed)
    }

    pub fn get_bytes_received(&self) -> u64 {
        self.bytes_received.load(Ordering::Relaxed)
    }

    pub fn get_total_connections(&self) -> u64 {
        self.connections_total.load(Ordering::Relaxed)
    }

    pub fn get_active_connections(&self) -> u64 {
        self.connections_active.load(Ordering::Relaxed)
    }
}

/// Event from the proxy listener
#[derive(Debug)]
pub enum ProxyEvent {
    /// New connection accepted
    NewConnection {
        id: u64,
        stream: TcpStream,
        addr: SocketAddr,
    },
    /// Connection closed
    ConnectionClosed { id: u64 },
    /// Error occurred
    Error { message: String },
}

/// Listens for incoming TCP connections to proxy through the tunnel
pub struct ProxyListener {
    /// Local port to listen on
    port: u16,

    /// The TCP listener
    listener: Option<TcpListener>,

    /// Event channel sender
    event_tx: mpsc::Sender<ProxyEvent>,

    /// Event channel receiver
    event_rx: mpsc::Receiver<ProxyEvent>,

    /// Connection counter
    connection_counter: AtomicU64,

    /// Statistics
    stats: Arc<ProxyStats>,

    /// Running flag
    running: bool,
}

impl ProxyListener {
    /// Create a new proxy listener
    pub fn new(port: u16) -> Self {
        let (event_tx, event_rx) = mpsc::channel(64);

        Self {
            port,
            listener: None,
            event_tx,
            event_rx,
            connection_counter: AtomicU64::new(0),
            stats: ProxyStats::new(),
            running: false,
        }
    }

    /// Get the port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get statistics
    pub fn stats(&self) -> Arc<ProxyStats> {
        Arc::clone(&self.stats)
    }

    /// Take the event receiver
    pub fn take_event_receiver(&mut self) -> mpsc::Receiver<ProxyEvent> {
        let (new_tx, new_rx) = mpsc::channel(64);
        let old_rx = std::mem::replace(&mut self.event_rx, new_rx);
        self.event_tx = new_tx;
        old_rx
    }

    /// Start listening
    pub async fn start(&mut self) -> Result<SocketAddr> {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).await.map_err(|e| {
            RiftError::PortBindFailed {
                port: self.port,
                reason: e.to_string(),
            }
        })?;

        let local_addr = listener.local_addr()?;
        self.port = local_addr.port();
        self.listener = Some(listener);
        self.running = true;

        info!("Proxy listening on {}", local_addr);

        Ok(local_addr)
    }

    /// Run the accept loop
    pub async fn run(&mut self) -> Result<()> {
        let listener = self
            .listener
            .take()
            .ok_or_else(|| RiftError::PortBindFailed {
                port: self.port,
                reason: "Listener not started".to_string(),
            })?;

        let event_tx = self.event_tx.clone();
        let stats = Arc::clone(&self.stats);
        let counter = &self.connection_counter;

        while self.running {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            let id = counter.fetch_add(1, Ordering::Relaxed);
                            stats.connection_opened();

                            debug!("Accepted connection {} from {}", id, addr);

                            let _ = event_tx
                                .send(ProxyEvent::NewConnection { id, stream, addr })
                                .await;
                        }
                        Err(e) => {
                            error!("Accept error: {}", e);
                            let _ = event_tx
                                .send(ProxyEvent::Error {
                                    message: e.to_string(),
                                })
                                .await;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Stop the listener
    pub fn stop(&mut self) {
        self.running = false;
    }
}

/// Helper to create a local listener that mirrors a remote port
pub async fn create_local_mirror(port: u16) -> Result<ProxyListener> {
    let mut listener = ProxyListener::new(port);
    listener.start().await?;
    Ok(listener)
}
