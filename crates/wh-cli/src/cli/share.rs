//! Share Command Implementation

use anyhow::Result;
use wh_core::PortKeyConfig;
use wh_daemon::{DaemonCommand, DaemonServer};
use std::path::PathBuf;
use tracing::info;

use crate::tui;

/// Run the share command
pub async fn run(port: u16, secrets: Option<PathBuf>, no_tui: bool) -> Result<()> {
    info!("Sharing port {} (secrets: {:?})", port, secrets);

    // Create daemon
    let config = PortKeyConfig::default();
    let mut daemon = DaemonServer::new(config).await?;

    // Get handles
    let command_tx = daemon.command_sender();
    let event_rx = daemon.take_event_receiver();

    // Start the daemon
    daemon.start().await?;

    // Create share session
    command_tx
        .send(DaemonCommand::Share {
            port,
            secrets_path: secrets,
        })
        .await?;

    // Get the link
    let link = daemon.portkey_link().await;
    
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                     🔑 PortKey Share                         ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ Sharing: localhost:{}                                       ║", port);
    println!("║                                                              ║");
    println!("║ Share this link with peers:                                  ║");
    println!("║ {}  ║", format!("{:<54}", link));
    println!("║                                                              ║");
    println!("║ Waiting for connections...                                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    if no_tui {
        // Simple mode - just run the daemon
        daemon.run().await?;
    } else {
        // Run with TUI
        tui::run_share_tui(port, link, event_rx, command_tx).await?;
    }

    Ok(())
}
