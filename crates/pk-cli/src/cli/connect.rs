//! Connect Command Implementation

use anyhow::Result;
use pk_core::{PortKeyConfig, PeerId, secrets::{EnvVault, SecretsRequest}};
use pk_daemon::{DaemonCommand, DaemonServer};
use std::path::PathBuf;
use tracing::{info, error};

use crate::tui;

/// Run the connect command
pub async fn run(
    link: String,
    local_port: Option<u16>,
    request_secrets: bool,
    save_secrets: Option<PathBuf>,
    no_tui: bool,
) -> Result<()> {
    // Ensure link has the pk:// prefix
    let link = if link.starts_with("pk://") {
        link
    } else {
        format!("pk://{}", link)
    };

    // Extract port from link if present (format: pk://PEER_ID/PORT)
    let (peer_link, port) = if let Some(idx) = link.rfind('/') {
        let port_str = &link[idx + 1..];
        if let Ok(p) = port_str.parse::<u16>() {
            (link[..idx].to_string(), p)
        } else {
            // No port in link, use default 3000
            (link.clone(), 3000)
        }
    } else {
        (link.clone(), 3000)
    };

    let local_port = local_port.unwrap_or(port);

    info!("Connecting to {} port {} (local: {})", peer_link, port, local_port);

    // Create daemon
    let config = PortKeyConfig::default();
    let mut daemon = DaemonServer::new(config).await?;

    // Get handles
    let command_tx = daemon.command_sender();
    let event_rx = daemon.take_event_receiver();

    // Start the daemon
    daemon.start().await?;

    // Create connect session
    command_tx
        .send(DaemonCommand::Connect {
            link: peer_link.clone(),
            port,
            local_port: Some(local_port),
        })
        .await?;

    // Request secrets if flag is set
    if request_secrets {
        if let Err(e) = request_secrets_from_peer(&peer_link, &save_secrets).await {
            error!("Failed to request secrets: {}", e);
            eprintln!("⚠️  Failed to retrieve secrets: {}", e);
        }
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    🔗 PortKey Connect                        ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ Connecting to: {}  ║", format!("{:<42}", peer_link));
    println!("║ Remote port: {}                                              ║", port);
    println!("║ Local port:  {}                                              ║", local_port);
    println!("║                                                              ║");
    println!("║ Access the tunnel at: http://localhost:{}                   ║", local_port);
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    if no_tui {
        // Simple mode - just run the daemon
        daemon.run().await?;
    } else {
        // Run with TUI
        tui::run_connect_tui(peer_link, port, local_port, event_rx, command_tx).await?;
    }

    Ok(())
}

/// Request secrets from a peer
async fn request_secrets_from_peer(
    peer_link: &str,
    save_path: &Option<PathBuf>,
) -> Result<()> {
    use pk_core::{network::{PeerIdentity, SECRETS_PROTOCOL}, send_secrets, receive_secrets};
    use tokio_util::compat::FuturesAsyncReadCompatExt;
    
    info!("Requesting secrets from peer");
    
    // Create a temporary network just for secrets request
    let config = PortKeyConfig::default();
    let mut network = pk_core::PeerNetwork::new(config).await?;
    
    // Parse peer ID from link
    let peer_id: PeerId = PeerIdentity::parse_portkey_link(peer_link)?;
    
    // Connect to peer
    network.connect(peer_link).await?;
    info!("Connected to peer for secrets request");
    
    // Create our vault to get our public key
    let vault = EnvVault::from_file(".env.portkey.tmp")
        .unwrap_or_else(|_| {
            // If no file exists, create a new vault with identity
            let keypair = EnvVault::load_or_create_identity()
                .expect("Failed to load identity");
            EnvVault::with_keypair(keypair)
        });
    
    // Open a secrets stream
    let mut control = network.stream_control();
    let stream = control
        .open_stream(peer_id, SECRETS_PROTOCOL)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to open secrets stream: {:?}", e))?;
    
    info!("Opened secrets stream");
    
    // Split the stream
    let stream = stream.compat();
    let (mut read, mut write) = tokio::io::split(stream);
    
    // Send our public key in a SecretsRequest
    let request = SecretsRequest {
        public_key: vault.public_key().to_vec(),
    };
    
    send_secrets(&mut write, &request).await
        .map_err(|e| anyhow::anyhow!("Failed to send secrets request: {}", e))?;
    
    info!("Sent secrets request");
    
    // Receive the encrypted secrets response
    let response = receive_secrets(&mut read).await
        .map_err(|e| anyhow::anyhow!("Failed to receive secrets response: {}", e))?;
    
    info!("Received secrets response");
    
    // Decrypt the secrets
    let secrets = vault.decrypt_from_peer(&response)
        .map_err(|e| anyhow::anyhow!("Failed to decrypt secrets: {}", e))?;
    
    println!("\n🔐 Successfully received and decrypted shared secrets!");
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                      Shared Secrets                          ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    
    for (key, value) in &secrets {
        println!("║ {:<28} = {:<30} ║", key, value);
    }
    
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    
    // Save to file if requested
    if let Some(path) = save_path {
        let mut content = String::new();
        for (key, value) in &secrets {
            content.push_str(&format!("{}={}\n", key, value));
        }
        
        std::fs::write(path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write secrets to file: {}", e))?;
        
        println!("✅ Secrets saved to: {}", path.display());
    }
    
    Ok(())
}
