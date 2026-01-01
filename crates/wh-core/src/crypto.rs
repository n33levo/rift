//! Cryptographic utilities for PortKey
//!
//! Provides X25519 key exchange and AES-GCM encryption for secrets sharing.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::rngs::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::error::{PortKeyError, Result};

/// Size of the AES-GCM nonce
pub const NONCE_SIZE: usize = 12;

/// Size of the X25519 public key
pub const PUBLIC_KEY_SIZE: usize = 32;

/// Key pair for X25519 key exchange
#[derive(Clone)]
pub struct KeyPair {
    secret: StaticSecret,
    public: PublicKey,
}

impl KeyPair {
    /// Generate a new random key pair
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        Self { secret, public }
    }

    /// Create from existing secret bytes
    pub fn from_secret_bytes(bytes: [u8; 32]) -> Self {
        let secret = StaticSecret::from(bytes);
        let public = PublicKey::from(&secret);
        Self { secret, public }
    }

    /// Get the public key bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        *self.public.as_bytes()
    }

    /// Get the secret key bytes (for storage)
    pub fn secret_key_bytes(&self) -> [u8; 32] {
        self.secret.to_bytes()
    }

    /// Perform Diffie-Hellman key exchange with a peer's public key
    pub fn diffie_hellman(&self, peer_public: &[u8; 32]) -> [u8; 32] {
        let peer_public_key = PublicKey::from(*peer_public);
        *self.secret.diffie_hellman(&peer_public_key).as_bytes()
    }
}

impl std::fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyPair")
            .field("public", &BASE64.encode(self.public.as_bytes()))
            .finish()
    }
}

/// Ephemeral key exchange for one-time secrets sharing
pub struct EphemeralKeyExchange {
    secret: EphemeralSecret,
    public: PublicKey,
}

impl EphemeralKeyExchange {
    /// Generate a new ephemeral key pair
    pub fn new() -> Self {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        Self { secret, public }
    }

    /// Get the public key bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        *self.public.as_bytes()
    }

    /// Complete the key exchange and derive a shared secret
    pub fn complete(self, peer_public: &[u8; 32]) -> [u8; 32] {
        let peer_public_key = PublicKey::from(*peer_public);
        *self.secret.diffie_hellman(&peer_public_key).as_bytes()
    }
}

impl Default for EphemeralKeyExchange {
    fn default() -> Self {
        Self::new()
    }
}

/// AES-GCM encryption/decryption for secrets
pub struct SecretsCipher;

impl SecretsCipher {
    /// Encrypt data using AES-256-GCM with a shared secret
    pub fn encrypt(shared_secret: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, [u8; NONCE_SIZE])> {
        let cipher = Aes256Gcm::new_from_slice(shared_secret)
            .map_err(|e| PortKeyError::EncryptionFailed(e.to_string()))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::Rng::fill(&mut OsRng, &mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| PortKeyError::EncryptionFailed(e.to_string()))?;

        Ok((ciphertext, nonce_bytes))
    }

    /// Decrypt data using AES-256-GCM with a shared secret
    pub fn decrypt(
        shared_secret: &[u8; 32],
        ciphertext: &[u8],
        nonce: &[u8; NONCE_SIZE],
    ) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(shared_secret)
            .map_err(|e| PortKeyError::DecryptionFailed(e.to_string()))?;

        let nonce = Nonce::from_slice(nonce);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| PortKeyError::DecryptionFailed(e.to_string()))?;

        Ok(plaintext)
    }
}

/// Encrypt secrets for a specific recipient
pub fn encrypt_for_recipient(
    recipient_public_key: &[u8; 32],
    plaintext: &[u8],
) -> Result<(Vec<u8>, Vec<u8>, [u8; NONCE_SIZE])> {
    // Generate ephemeral key pair
    let ephemeral = EphemeralKeyExchange::new();
    let ephemeral_public = ephemeral.public_key_bytes();

    // Derive shared secret
    let shared_secret = ephemeral.complete(recipient_public_key);

    // Encrypt with shared secret
    let (ciphertext, nonce) = SecretsCipher::encrypt(&shared_secret, plaintext)?;

    Ok((ephemeral_public.to_vec(), ciphertext, nonce))
}

/// Decrypt secrets using our private key
pub fn decrypt_from_sender(
    our_keypair: &KeyPair,
    sender_ephemeral_public: &[u8; 32],
    ciphertext: &[u8],
    nonce: &[u8; NONCE_SIZE],
) -> Result<Vec<u8>> {
    // Derive shared secret
    let shared_secret = our_keypair.diffie_hellman(sender_ephemeral_public);

    // Decrypt
    SecretsCipher::decrypt(&shared_secret, ciphertext, nonce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate();
        let public = keypair.public_key_bytes();
        assert_eq!(public.len(), 32);
    }

    #[test]
    fn test_key_exchange() {
        let alice = KeyPair::generate();
        let bob = KeyPair::generate();

        let alice_shared = alice.diffie_hellman(&bob.public_key_bytes());
        let bob_shared = bob.diffie_hellman(&alice.public_key_bytes());

        assert_eq!(alice_shared, bob_shared);
    }

    #[test]
    fn test_encryption_roundtrip() {
        let shared_secret = [42u8; 32];
        let plaintext = b"Hello, PortKey!";

        let (ciphertext, nonce) = SecretsCipher::encrypt(&shared_secret, plaintext).unwrap();
        let decrypted = SecretsCipher::decrypt(&shared_secret, &ciphertext, &nonce).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_full_encryption_flow() {
        let recipient = KeyPair::generate();
        let plaintext = b"SECRET_KEY=super_secret_value";

        // Sender encrypts
        let (ephemeral_public, ciphertext, nonce) =
            encrypt_for_recipient(&recipient.public_key_bytes(), plaintext).unwrap();

        // Recipient decrypts
        let ephemeral_public: [u8; 32] = ephemeral_public.try_into().unwrap();
        let decrypted =
            decrypt_from_sender(&recipient, &ephemeral_public, &ciphertext, &nonce).unwrap();

        assert_eq!(decrypted, plaintext);
    }
}
