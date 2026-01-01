//! Info Command Implementation

use anyhow::Result;
use wh_core::{network::PeerIdentity, RiftConfig};

/// Run the info command
pub async fn run() -> Result<()> {
    let config = RiftConfig::default();
    let identity = PeerIdentity::load_or_generate(&config.identity_path)?;
    
    let peer_id = identity.peer_id().to_string();
    let link = identity.to_rift_link();
    let path = config.identity_path.display().to_string();
    
    println!("\n🔑 Rift Info\n");
    println!("Peer ID:");
    println!("  {}\n", peer_id);
    println!("Rift Link:");
    println!("  {}\n", link);
    println!("Identity Path:");
    println!("  {}\n", path);

    Ok(())
}
