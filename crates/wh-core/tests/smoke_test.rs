//! Simple smoke test to verify the tunnel functions compile and can be called
//!
//! This is a quick sanity check that doesn't require full P2P setup.

use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Test that we can run a simple TCP echo server
#[tokio::test]
async fn test_tcp_echo_server() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    // Spawn echo server
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 1024];
        let n = socket.read(&mut buf).await.unwrap();
        socket.write_all(&buf[..n]).await.unwrap();
    });
    
    // Connect and test
    let mut client = TcpStream::connect(addr).await.unwrap();
    client.write_all(b"Hello").await.unwrap();
    
    let mut buf = vec![0u8; 1024];
    let n = client.read(&mut buf).await.unwrap();
    
    assert_eq!(&buf[..n], b"Hello");
}

/// Test that bridge_stream_to_tcp function exists and is callable
#[tokio::test]
async fn test_bridge_function_signature() {
    // This just verifies the function exists and has the right signature
    let _ = wh_core::bridge_stream_to_tcp; 
    let _ = wh_core::open_tunnel_stream;
    
    // Verify PeerNetwork can be constructed (will fail without proper setup, that's OK)
    let config = wh_core::PortKeyConfig {
        listen_port: 0,
        identity_path: std::env::temp_dir().join("test_peer"),
        ..Default::default()
    };
    
    // This might fail if ports are in use, but at least it compiles
    let result = tokio::time::timeout(
        Duration::from_millis(100),
        wh_core::PeerNetwork::new(config)
    ).await;
    
    // We don't care if it times out or fails, we just want to verify it compiles
    let _ = result;
}

/// Test that the config can be created with defaults
#[test]
fn test_config_creation() {
    let config = wh_core::PortKeyConfig::default();
    assert_eq!(config.listen_port, 0);
}
