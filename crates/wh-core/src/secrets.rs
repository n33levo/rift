//! EnvVault - Secrets Management for PortKey
//!
//! Provides secure storage and sharing of environment variables.
//! Uses the system keyring for local storage and X25519/AES-GCM for transit.

use std::collections::HashMap;
use std::path::Path;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use keyring::Entry;
use serde::{Deserialize, Serialize};

use crate::crypto::{decrypt_from_sender, encrypt_for_recipient, KeyPair, NONCE_SIZE};
use crate::error::{PortKeyError, Result};

/// Service name for keyring storage
const KEYRING_SERVICE: &str = "portkey";

/// Key for storing the identity keypair
const IDENTITY_KEY: &str = "identity";

/// Request for secrets from a peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsRequest {
    /// Requester's public key
    pub public_key: Vec<u8>,
}

/// Response containing encrypted secrets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsResponse {
    /// Ephemeral public key used for encryption
    pub ephemeral_public_key: Vec<u8>,
    /// Encrypted secrets blob
    pub encrypted_data: Vec<u8>,
    /// Nonce used for encryption
    pub nonce: Vec<u8>,
    /// Sender's public key (for identification)
    pub sender_public_key: Vec<u8>,
}

/// EnvVault manages secrets for PortKey tunnels
#[derive(Debug, Clone)]
pub struct EnvVault {
    /// Environment variables to share
    secrets: HashMap<String, String>,

    /// Our keypair for encryption
    keypair: KeyPair,
}

impl EnvVault {
    /// Create a new EnvVault with a fresh keypair
    pub fn new() -> Self {
        Self {
            secrets: HashMap::new(),
            keypair: KeyPair::generate(),
        }
    }

    /// Create an EnvVault with an existing keypair
    pub fn with_keypair(keypair: KeyPair) -> Self {
        Self {
            secrets: HashMap::new(),
            keypair,
        }
    }

    /// Load or create identity keypair from system keyring
    pub fn load_or_create_identity() -> Result<KeyPair> {
        let entry = Entry::new(KEYRING_SERVICE, IDENTITY_KEY)
            .map_err(|e| PortKeyError::KeyringError(e.to_string()))?;

        // Try to load existing key
        match entry.get_password() {
            Ok(key_b64) => {
                let key_bytes = BASE64.decode(&key_b64)
                    .map_err(|e| PortKeyError::KeyringError(format!("Invalid key format: {}", e)))?;
                
                if key_bytes.len() != 32 {
                    return Err(PortKeyError::KeyringError("Invalid key length".to_string()));
                }

                let mut arr = [0u8; 32];
                arr.copy_from_slice(&key_bytes);
                Ok(KeyPair::from_secret_bytes(arr))
            }
            Err(_) => {
                // Generate new keypair and store it
                let keypair = KeyPair::generate();
                let key_b64 = BASE64.encode(keypair.secret_key_bytes());
                
                entry
                    .set_password(&key_b64)
                    .map_err(|e| PortKeyError::KeyringError(e.to_string()))?;

                Ok(keypair)
            }
        }
    }

    /// Get our public key bytes
    pub fn public_key(&self) -> [u8; 32] {
        self.keypair.public_key_bytes()
    }

    /// Create a new EnvVault from a .env file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let keypair = Self::load_or_create_identity()?;
        let mut vault = Self::with_keypair(keypair);
        vault.load_env_file(path)?;
        Ok(vault)
    }

    /// Load secrets from a .env file
    pub fn load_env_file(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(PortKeyError::EnvParseError(format!(
                "File not found: {}",
                path.display()
            )));
        }

        let contents = std::fs::read_to_string(path)?;
        self.parse_env_contents(&contents)?;

        Ok(())
    }

    /// Parse .env file contents
    fn parse_env_contents(&mut self, contents: &str) -> Result<()> {
        for (line_num, line) in contents.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse KEY=VALUE
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = Self::parse_value(value.trim());

                if key.is_empty() {
                    return Err(PortKeyError::EnvParseError(format!(
                        "Empty key at line {}",
                        line_num + 1
                    )));
                }

                self.secrets.insert(key, value);
            } else {
                return Err(PortKeyError::EnvParseError(format!(
                    "Invalid format at line {}: expected KEY=VALUE",
                    line_num + 1
                )));
            }
        }

        Ok(())
    }

    /// Parse a value, handling quotes
    fn parse_value(value: &str) -> String {
        let value = value.trim();

        // Handle quoted values
        if (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
        {
            if value.len() >= 2 {
                return value[1..value.len() - 1].to_string();
            }
        }

        value.to_string()
    }

    /// Add a secret
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.secrets.insert(key.into(), value.into());
    }

    /// Get a secret
    pub fn get(&self, key: &str) -> Option<&String> {
        self.secrets.get(key)
    }

    /// Remove a secret
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.secrets.remove(key)
    }

    /// Get all secrets
    pub fn secrets(&self) -> &HashMap<String, String> {
        &self.secrets
    }

    /// Check if there are any secrets
    pub fn is_empty(&self) -> bool {
        self.secrets.is_empty()
    }

    /// Number of secrets
    pub fn len(&self) -> usize {
        self.secrets.len()
    }

    /// Create a secrets request message
    pub fn create_secrets_request(&self) -> SecretsRequest {
        SecretsRequest {
            public_key: self.keypair.public_key_bytes().to_vec(),
        }
    }

    /// Encrypt secrets for a requesting peer
    pub fn encrypt_for_peer(&self, peer_public_key: &[u8]) -> Result<SecretsResponse> {
        if peer_public_key.len() != 32 {
            return Err(PortKeyError::InvalidPublicKey(
                "Public key must be 32 bytes".to_string(),
            ));
        }

        let mut peer_key = [0u8; 32];
        peer_key.copy_from_slice(peer_public_key);

        // Serialize secrets
        let secrets_json = serde_json::to_vec(&self.secrets)?;

        // Encrypt
        let (ephemeral_public, encrypted_data, nonce) =
            encrypt_for_recipient(&peer_key, &secrets_json)?;

        Ok(SecretsResponse {
            ephemeral_public_key: ephemeral_public,
            encrypted_data,
            nonce: nonce.to_vec(),
            sender_public_key: self.keypair.public_key_bytes().to_vec(),
        })
    }

    /// Decrypt secrets from a peer's response
    pub fn decrypt_from_peer(&self, response: &SecretsResponse) -> Result<HashMap<String, String>> {
        if response.ephemeral_public_key.len() != 32 {
            return Err(PortKeyError::InvalidPublicKey(
                "Ephemeral public key must be 32 bytes".to_string(),
            ));
        }

        if response.nonce.len() != NONCE_SIZE {
            return Err(PortKeyError::DecryptionFailed(
                "Invalid nonce size".to_string(),
            ));
        }

        let mut ephemeral_key = [0u8; 32];
        ephemeral_key.copy_from_slice(&response.ephemeral_public_key);

        let mut nonce = [0u8; NONCE_SIZE];
        nonce.copy_from_slice(&response.nonce);

        let plaintext = decrypt_from_sender(
            &self.keypair,
            &ephemeral_key,
            &response.encrypted_data,
            &nonce,
        )?;

        let secrets: HashMap<String, String> = serde_json::from_slice(&plaintext)?;
        Ok(secrets)
    }

    /// Export secrets to .env format string
    pub fn to_env_format(&self) -> String {
        let mut lines = Vec::new();
        
        for (key, value) in &self.secrets {
            // Quote values that contain spaces or special characters
            if value.contains(' ') || value.contains('"') || value.contains('\'') {
                lines.push(format!("{}=\"{}\"", key, value.replace('"', "\\\"")));
            } else {
                lines.push(format!("{}={}", key, value));
            }
        }

        lines.sort(); // Consistent ordering
        lines.join("\n")
    }

    /// Write secrets to a temporary file
    pub fn write_to_temp_file(&self) -> Result<std::path::PathBuf> {
        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path().join(".env.portkey");
        
        std::fs::write(&temp_path, self.to_env_format())?;
        
        // Keep the temp directory around (don't delete on drop)
        std::mem::forget(temp_dir);
        
        Ok(temp_path)
    }

    /// Inject secrets into the current process environment
    /// 
    /// # Safety
    /// This function modifies environment variables which is unsafe in multi-threaded contexts.
    /// Only call this from a single-threaded context or when you know no other threads are
    /// reading environment variables.
    pub fn inject_into_env(&self) {
        for (key, value) in &self.secrets {
            // SAFETY: We assume this is called before spawning threads that read env vars
            unsafe {
                std::env::set_var(key, value);
            }
        }
    }
}

impl Default for EnvVault {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable version of secrets for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSecrets {
    pub secrets: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env() {
        let mut vault = EnvVault::new();
        
        let contents = r#"
            # Comment line
            DATABASE_URL=postgres://localhost:5432/db
            SECRET_KEY="my secret key"
            EMPTY_VALUE=
            SINGLE_QUOTED='value'
        "#;

        vault.parse_env_contents(contents).unwrap();

        assert_eq!(
            vault.get("DATABASE_URL"),
            Some(&"postgres://localhost:5432/db".to_string())
        );
        assert_eq!(
            vault.get("SECRET_KEY"),
            Some(&"my secret key".to_string())
        );
        assert_eq!(vault.get("EMPTY_VALUE"), Some(&"".to_string()));
        assert_eq!(vault.get("SINGLE_QUOTED"), Some(&"value".to_string()));
    }

    #[test]
    fn test_encryption_roundtrip() {
        let sender_vault = EnvVault::new();
        let receiver_vault = EnvVault::new();

        let mut vault = EnvVault::with_keypair(sender_vault.keypair.clone());
        vault.set("API_KEY", "secret123");
        vault.set("DATABASE_URL", "postgres://localhost/db");

        // Sender encrypts for receiver
        let response = vault.encrypt_for_peer(&receiver_vault.public_key()).unwrap();

        // Receiver decrypts
        let decrypted = receiver_vault.decrypt_from_peer(&response).unwrap();

        assert_eq!(decrypted.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(
            decrypted.get("DATABASE_URL"),
            Some(&"postgres://localhost/db".to_string())
        );
    }

    #[test]
    fn test_to_env_format() {
        let mut vault = EnvVault::new();
        vault.set("KEY1", "value1");
        vault.set("KEY2", "value with spaces");

        let output = vault.to_env_format();
        assert!(output.contains("KEY1=value1"));
        assert!(output.contains("KEY2=\"value with spaces\""));
    }
}
