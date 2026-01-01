//! Tunnel Proxy Implementation
//!
//! Manages bidirectional data flow between local TCP connections
//! and remote peers over the P2P network.

use bytes::{Bytes, BytesMut};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info};

use super::listener::ProxyStats;
use crate::error::{PortKeyError, Result};
use crate::protocol::{DataFrame, Message, MessagePayload};

/// Buffer size for reading from TCP connections
const BUFFER_SIZE: usize = 64 * 1024; // 64KB

/// Command to send to the tunnel
#[derive(Debug)]
pub enum TunnelCommand {
    /// Send data to the remote peer
    SendData { stream_id: u64, data: Vec<u8> },
    /// Close a stream
    CloseStream { stream_id: u64 },
    /// Shutdown the tunnel
    Shutdown,
}

/// Event from the tunnel
#[derive(Debug, Clone)]
pub enum TunnelEvent {
    /// Data received from remote
    DataReceived { stream_id: u64, data: Vec<u8> },
    /// Stream closed
    StreamClosed { stream_id: u64 },
    /// Tunnel closed
    TunnelClosed,
}

/// Represents an active stream/connection
struct ActiveStream {
    /// Sender to forward data to the TCP connection
    data_tx: mpsc::Sender<Bytes>,
    /// Whether the stream is still active
    #[allow(dead_code)]
    active: bool,
}

/// Manages the tunnel between local TCP and remote peer
pub struct TunnelProxy {
    /// Target port on the sharing peer
    target_port: u16,

    /// Stream counter for unique IDs
    stream_counter: AtomicU64,

    /// Active streams
    streams: Arc<RwLock<HashMap<u64, ActiveStream>>>,

    /// Command sender
    command_tx: mpsc::Sender<TunnelCommand>,

    /// Command receiver
    command_rx: mpsc::Receiver<TunnelCommand>,

    /// Event sender (to network layer)
    #[allow(dead_code)]
    event_tx: mpsc::Sender<TunnelEvent>,

    /// Statistics
    stats: Arc<ProxyStats>,

    /// Running flag
    running: bool,
}

impl TunnelProxy {
    /// Create a new tunnel proxy
    pub fn new(target_port: u16) -> (Self, mpsc::Receiver<TunnelEvent>) {
        let (command_tx, command_rx) = mpsc::channel(256);
        let (event_tx, event_rx) = mpsc::channel(256);

        let proxy = Self {
            target_port,
            stream_counter: AtomicU64::new(0),
            streams: Arc::new(RwLock::new(HashMap::new())),
            command_tx,
            command_rx,
            event_tx,
            stats: ProxyStats::new(),
            running: true,
        };

        (proxy, event_rx)
    }

    /// Get the command sender
    pub fn command_sender(&self) -> mpsc::Sender<TunnelCommand> {
        self.command_tx.clone()
    }

    /// Get statistics
    pub fn stats(&self) -> Arc<ProxyStats> {
        Arc::clone(&self.stats)
    }

    /// Get the target port
    pub fn target_port(&self) -> u16 {
        self.target_port
    }

    /// Handle a new incoming TCP connection (from local side)
    pub async fn handle_local_connection(&self, stream: TcpStream) -> Result<u64> {
        let stream_id = self.stream_counter.fetch_add(1, Ordering::Relaxed);

        let (data_tx, data_rx) = mpsc::channel::<Bytes>(64);

        // Register the stream
        {
            let mut streams = self.streams.write().await;
            streams.insert(
                stream_id,
                ActiveStream {
                    data_tx,
                    active: true,
                },
            );
        }

        // Spawn task to handle this connection
        let streams = Arc::clone(&self.streams);
        let command_tx = self.command_tx.clone();
        let stats = Arc::clone(&self.stats);

        tokio::spawn(async move {
            if let Err(e) = Self::run_stream_handler(stream_id, stream, data_rx, command_tx, stats).await
            {
                error!("Stream {} handler error: {}", stream_id, e);
            }

            // Clean up
            streams.write().await.remove(&stream_id);
        });

        self.stats.connection_opened();

        Ok(stream_id)
    }

    /// Run the stream handler for a single connection
    async fn run_stream_handler(
        stream_id: u64,
        mut tcp_stream: TcpStream,
        mut data_rx: mpsc::Receiver<Bytes>,
        command_tx: mpsc::Sender<TunnelCommand>,
        stats: Arc<ProxyStats>,
    ) -> Result<()> {
        let (mut reader, mut writer) = tcp_stream.split();
        let mut buf = BytesMut::with_capacity(BUFFER_SIZE);

        loop {
            tokio::select! {
                // Read from TCP, send to tunnel
                result = reader.read_buf(&mut buf) => {
                    match result {
                        Ok(0) => {
                            // Connection closed
                            debug!("Stream {} TCP connection closed", stream_id);
                            let _ = command_tx
                                .send(TunnelCommand::CloseStream { stream_id })
                                .await;
                            break;
                        }
                        Ok(n) => {
                            let data = buf.split_to(n).to_vec();
                            stats.record_sent(n as u64);

                            let _ = command_tx
                                .send(TunnelCommand::SendData {
                                    stream_id,
                                    data,
                                })
                                .await;
                        }
                        Err(e) => {
                            error!("Stream {} read error: {}", stream_id, e);
                            break;
                        }
                    }
                }

                // Receive from tunnel, write to TCP
                Some(data) = data_rx.recv() => {
                    stats.record_received(data.len() as u64);

                    if let Err(e) = writer.write_all(&data).await {
                        error!("Stream {} write error: {}", stream_id, e);
                        break;
                    }
                }
            }
        }

        stats.connection_closed();
        Ok(())
    }

    /// Handle data received from the remote peer
    pub async fn handle_remote_data(&self, stream_id: u64, data: Vec<u8>) -> Result<()> {
        let streams = self.streams.read().await;

        if let Some(stream) = streams.get(&stream_id) {
            if stream.active {
                stream
                    .data_tx
                    .send(Bytes::from(data))
                    .await
                    .map_err(|_| PortKeyError::StreamError("Failed to send to stream".to_string()))?;
            }
        }

        Ok(())
    }

    /// Close a specific stream
    pub async fn close_stream(&self, stream_id: u64) {
        let mut streams = self.streams.write().await;
        if let Some(stream) = streams.get_mut(&stream_id) {
            stream.active = false;
        }
        streams.remove(&stream_id);
    }

    /// Process tunnel commands
    pub async fn process_commands<F>(&mut self, mut send_to_peer: F) -> Result<()>
    where
        F: FnMut(Message) -> Result<()>,
    {
        while let Some(command) = self.command_rx.recv().await {
            match command {
                TunnelCommand::SendData { stream_id, data } => {
                    let sequence = 0; // TODO: implement proper sequencing
                    let frame = DataFrame::new(stream_id, sequence, data);
                    let message = Message::new(stream_id, MessagePayload::DataFrame(frame));
                    send_to_peer(message)?;
                }
                TunnelCommand::CloseStream { stream_id } => {
                    self.close_stream(stream_id).await;
                }
                TunnelCommand::Shutdown => {
                    self.running = false;
                    break;
                }
            }
        }

        Ok(())
    }

    /// Check if the tunnel is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Shutdown the tunnel
    pub async fn shutdown(&mut self) {
        info!("Shutting down tunnel proxy");
        self.running = false;

        // Close all streams
        let stream_ids: Vec<u64> = self.streams.read().await.keys().cloned().collect();
        for id in stream_ids {
            self.close_stream(id).await;
        }
    }

    /// Get active stream count
    pub async fn active_stream_count(&self) -> usize {
        self.streams.read().await.len()
    }
}

/// Factory for creating tunnel proxies
pub struct TunnelProxyFactory;

impl TunnelProxyFactory {
    /// Create a tunnel proxy for sharing a local port
    pub fn create_sharing_proxy(port: u16) -> (TunnelProxy, mpsc::Receiver<TunnelEvent>) {
        TunnelProxy::new(port)
    }

    /// Create a tunnel proxy for connecting to a remote port
    pub fn create_connecting_proxy(port: u16) -> (TunnelProxy, mpsc::Receiver<TunnelEvent>) {
        TunnelProxy::new(port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tunnel_proxy_creation() {
        let (proxy, _rx) = TunnelProxy::new(3000);
        assert_eq!(proxy.target_port(), 3000);
        assert!(proxy.is_running());
    }

    #[tokio::test]
    async fn test_stream_counter() {
        let (proxy, _rx) = TunnelProxy::new(3000);

        let id1 = proxy.stream_counter.fetch_add(1, Ordering::Relaxed);
        let id2 = proxy.stream_counter.fetch_add(1, Ordering::Relaxed);

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
    }
}
