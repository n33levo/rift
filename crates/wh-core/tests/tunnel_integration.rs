//! Integration test for end-to-end tunnel functionality
//!
//! This test verifies the complete data path:
//! Client -> Local TCP (8080) -> Peer B -> QUIC Stream -> Peer A -> Target TCP (3000)

use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

/// Simple target server that responds with a known message
async fn start_target_server(port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .expect("Failed to bind target server");
        
        println!("[Target Server] Listening on 127.0.0.1:{}", port);
        
        loop {
            match listener.accept().await {
                Ok((mut socket, addr)) => {
                    println!("[Target Server] Accepted connection from {}", addr);
                    tokio::spawn(async move {
                        let mut buf = vec![0u8; 1024];
                        match socket.read(&mut buf).await {
                            Ok(n) if n > 0 => {
                                let request = String::from_utf8_lossy(&buf[..n]);
                                println!("[Target Server] Received: {}", request.trim());
                                
                                // Send response
                                let response = b"HTTP/1.1 200 OK\r\nContent-Length: 18\r\n\r\nHello from Target!";
                                if let Err(e) = socket.write_all(response).await {
                                    eprintln!("[Target Server] Failed to send response: {}", e);
                                }
                                println!("[Target Server] Sent response");
                            }
                            Ok(_) => println!("[Target Server] Client disconnected"),
                            Err(e) => eprintln!("[Target Server] Read error: {}", e),
                        }
                    });
                }
                Err(e) => eprintln!("[Target Server] Accept error: {}", e),
            }
        }
    })
}

/// Test client that connects to the tunnel and verifies the response
async fn test_client(port: u16) -> Result<String, Box<dyn std::error::Error>> {
    println!("[Test Client] Connecting to 127.0.0.1:{}", port);
    
    let mut stream = timeout(
        Duration::from_secs(2),
        TcpStream::connect(format!("127.0.0.1:{}", port))
    ).await??;
    
    println!("[Test Client] Connected, sending HTTP request");
    
    // Send HTTP request
    stream.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n").await?;
    
    // Read response
    let mut buf = vec![0u8; 1024];
    let n = timeout(Duration::from_secs(2), stream.read(&mut buf)).await??;
    
    let response = String::from_utf8_lossy(&buf[..n]).to_string();
    println!("[Test Client] Received {} bytes: {}", n, response.trim());
    
    Ok(response)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore] // Ignore by default since it requires full network setup
async fn test_end_to_end_tunnel() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for debugging
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info,wh_core=debug,wh_daemon=debug")
        .try_init();
    
    println!("\n=== Starting End-to-End Tunnel Test ===\n");
    
    // Step 1: Start target server on port 3000
    println!("[Setup] Starting target server on port 3000");
    let target_server = start_target_server(3000).await;
    tokio::time::sleep(Duration::from_millis(500)).await; // Let it bind
    
    // Step 2: Start Peer A (Sharer) - shares port 3000
    println!("[Setup] Starting Peer A (sharer)");
    let peer_a_config = wh_core::PortKeyConfig {
        listen_port: 9001,
        identity_path: std::path::PathBuf::from("/tmp/portkey_test_peer_a"),
        ..Default::default()
    };
    
    let mut peer_a_network = wh_core::PeerNetwork::new(peer_a_config).await?;
    peer_a_network.start_listening().await?;
    let peer_a_link = peer_a_network.portkey_link();
    
    println!("[Peer A] Link: {}", peer_a_link);
    println!("[Peer A] Started listening");
    
    // Spawn task to handle incoming streams on Peer A
    let mut peer_a_incoming = peer_a_network.take_incoming_streams();
    let peer_a_handler = tokio::spawn(async move {
        use futures::StreamExt;
        println!("[Peer A] Waiting for incoming streams...");
        while let Some((_peer_id, stream)) = peer_a_incoming.next().await {
            println!("[Peer A] Incoming stream! Bridging to localhost:3000");
            tokio::spawn(async move {
                if let Err(e) = wh_core::bridge_stream_to_tcp(stream, 3000).await {
                    eprintln!("[Peer A] Bridge error: {}", e);
                } else {
                    println!("[Peer A] Bridge completed successfully");
                }
            });
        }
    });
    
    // Keep peer A's network alive
    let peer_a_poll = tokio::spawn(async move {
        loop {
            peer_a_network.poll_once().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });
    
    // Step 3: Start Peer B (Connector) - connects to Peer A, listens on 8080
    println!("[Setup] Starting Peer B (connector)");
    let peer_b_config = wh_core::PortKeyConfig {
        listen_port: 9002,
        identity_path: std::path::PathBuf::from("/tmp/portkey_test_peer_b"),
        ..Default::default()
    };
    
    let mut peer_b_network = wh_core::PeerNetwork::new(peer_b_config).await?;
    peer_b_network.start_listening().await?;
    
    println!("[Peer B] Connecting to Peer A: {}", peer_a_link);
    let peer_a_id = peer_b_network.connect(&peer_a_link).await?;
    
    // Give time for connection to establish
    println!("[Peer B] Waiting for connection to establish...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Start local listener on Peer B side (port 8080)
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("[Peer B] Local listener started on 127.0.0.1:8080");
    
    // Spawn task to handle incoming TCP connections on Peer B
    let stream_control = peer_b_network.stream_control();
    let peer_b_handler = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((tcp_stream, addr)) => {
                    println!("[Peer B] Incoming TCP connection from {}", addr);
                    let mut control = stream_control.clone();
                    let peer_id = peer_a_id;
                    
                    tokio::spawn(async move {
                        use tokio_util::compat::FuturesAsyncReadCompatExt;
                        
                        match wh_core::open_tunnel_stream(&mut control, peer_id).await {
                            Ok(stream) => {
                                println!("[Peer B] Opened stream to peer, starting bridge");
                                let stream = stream.compat();
                                let (mut stream_read, mut stream_write) = tokio::io::split(stream);
                                let (mut tcp_read, mut tcp_write) = tcp_stream.into_split();
                                
                                // Bidirectional copy
                                tokio::select! {
                                    r = tokio::io::copy(&mut stream_read, &mut tcp_write) => {
                                        if let Err(e) = r {
                                            eprintln!("[Peer B] Stream->TCP error: {}", e);
                                        } else {
                                            println!("[Peer B] Stream->TCP completed");
                                        }
                                    }
                                    r = tokio::io::copy(&mut tcp_read, &mut stream_write) => {
                                        if let Err(e) = r {
                                            eprintln!("[Peer B] TCP->Stream error: {}", e);
                                        } else {
                                            println!("[Peer B] TCP->Stream completed");
                                        }
                                    }
                                }
                                println!("[Peer B] Bridge completed");
                            }
                            Err(e) => {
                                eprintln!("[Peer B] Failed to open stream: {}", e);
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("[Peer B] Accept error: {}", e);
                    break;
                }
            }
        }
    });
    
    // Keep peer B's network alive
    let peer_b_poll = tokio::spawn(async move {
        loop {
            peer_b_network.poll_once().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });
    
    // Step 4: Give everything time to stabilize
    println!("\n[Test] Waiting for tunnel to stabilize...");
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Step 5: Send test request through the tunnel
    println!("\n[Test] Sending request through tunnel (localhost:8080 -> Peer B -> Peer A -> localhost:3000)");
    
    let response = timeout(
        Duration::from_secs(5),
        test_client(8080)
    ).await??;
    
    // Step 6: Verify response
    println!("\n[Test] Verifying response...");
    assert!(
        response.contains("Hello from Target"),
        "Expected 'Hello from Target' in response, got: {}",
        response
    );
    
    println!("\n✅ Test PASSED! Tunnel is working end-to-end!\n");
    
    // Cleanup
    target_server.abort();
    peer_a_handler.abort();
    peer_a_poll.abort();
    peer_b_handler.abort();
    peer_b_poll.abort();
    
    Ok(())
}

/// Simpler unit-style test to verify bridge_stream_to_tcp compiles and has correct signature
#[tokio::test]
async fn test_bridge_function_exists() {
    // This just ensures the function exists and can be called
    // We can't easily test it without a full libp2p setup
    let _ = wh_core::bridge_stream_to_tcp; // Function exists
    let _ = wh_core::open_tunnel_stream; // Function exists
}
