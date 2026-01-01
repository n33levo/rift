//! Info Command Implementation

use anyhow::Result;
use wh_core::{network::PeerIdentity, PortKeyConfig};

/// Run the info command
pub async fn run() -> Result<()> {
    let config = PortKeyConfig::default();
    let identity = PeerIdentity::load_or_generate(&config.identity_path)?;

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                      🔑 PortKey Info                         ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ Peer ID:                                                     ║");
    println!("║   {}  ║", identity.peer_id());
    println!("║                                                              ║");
    println!("║ PortKey Link:                                                ║");
    println!("║   {}  ║", identity.to_portkey_link());
    println!("║                                                              ║");
    println!("║ Identity Path:                                               ║");
    println!("║   {}  ║", format!("{:<52}", config.identity_path.display()));
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    Ok(())
}
