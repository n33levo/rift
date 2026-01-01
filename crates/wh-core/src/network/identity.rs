//! Peer Identity Management
//!
//! Handles libp2p keypair generation, storage, and peer ID derivation.

use libp2p::identity::{Keypair, PeerId};
use std::path::Path;

use crate::error::{RiftError, Result};

/// Manages the peer identity (keypair and peer ID)
#[derive(Clone)]
pub struct PeerIdentity {
    keypair: Keypair,
    peer_id: PeerId,
}

impl PeerIdentity {
    /// Generate a new random identity
    pub fn generate() -> Self {
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        Self { keypair, peer_id }
    }

    /// Load identity from a file or generate a new one
    pub fn load_or_generate(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if path.exists() {
            Self::load(path)
        } else {
            let identity = Self::generate();
            identity.save(path)?;
            Ok(identity)
        }
    }

    /// Load identity from a file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let bytes = std::fs::read(path.as_ref())?;

        // Try to decode as protobuf-encoded keypair
        let keypair = Keypair::from_protobuf_encoding(&bytes)
            .map_err(|e| RiftError::ConfigError(format!("Invalid keypair file: {}", e)))?;

        let peer_id = PeerId::from(keypair.public());

        Ok(Self { keypair, peer_id })
    }

    /// Save identity to a file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let bytes = self
            .keypair
            .to_protobuf_encoding()
            .map_err(|e| RiftError::ConfigError(format!("Failed to encode keypair: {}", e)))?;

        std::fs::write(path, bytes)?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Get the keypair
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    /// Get the peer ID
    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    /// Get the peer ID as a string
    pub fn peer_id_string(&self) -> String {
        self.peer_id.to_string()
    }

    /// Generate a Rift link for sharing
    pub fn to_rift_link(&self) -> String {
        format!("rift://{}", self.peer_id)
    }

    /// Parse a peer ID from a Rift link
    pub fn parse_rift_link(link: &str) -> Result<PeerId> {
        let peer_id_str = link
            .strip_prefix("rift://")
            .ok_or_else(|| RiftError::InvalidPeerId("Link must start with rift://".to_string()))?;

        peer_id_str
            .parse()
            .map_err(|e| RiftError::InvalidPeerId(format!("Invalid peer ID: {}", e)))
    }
}

impl std::fmt::Debug for PeerIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PeerIdentity")
            .field("peer_id", &self.peer_id.to_string())
            .finish()
    }
}

impl std::fmt::Display for PeerIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.peer_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_identity_generation() {
        let identity = PeerIdentity::generate();
        assert!(!identity.peer_id_string().is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("identity.key");

        let original = PeerIdentity::generate();
        original.save(&path).unwrap();

        let loaded = PeerIdentity::load(&path).unwrap();
        assert_eq!(original.peer_id(), loaded.peer_id());
    }

    #[test]
    fn test_rift_link() {
        let identity = PeerIdentity::generate();
        let link = identity.to_rift_link();
        
        assert!(link.starts_with("rift://"));
        
        let parsed = PeerIdentity::parse_rift_link(&link).unwrap();
        assert_eq!(*identity.peer_id(), parsed);
    }
}
