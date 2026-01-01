//! Secrets Encryption Test
//!
//! Verifies that EnvVault can encrypt and decrypt secrets correctly.

use wh_core::secrets::EnvVault;

#[test]
fn test_secrets_encryption_roundtrip() {
    // Create two vaults (simulating two peers)
    let mut vault_a = EnvVault::new();
    let vault_b = EnvVault::new();

    // Add some secrets to vault A
    vault_a.set("DATABASE_URL", "postgres://localhost/mydb");
    vault_a.set("API_KEY", "super-secret-key-12345");
    vault_a.set("DEBUG", "true");

    // Get B's public key
    let peer_b_public_key = vault_b.public_key();

    // Encrypt secrets for peer B
    let encrypted_response = vault_a
        .encrypt_for_peer(&peer_b_public_key)
        .expect("Failed to encrypt secrets");

    // Verify encrypted data is not empty
    assert!(!encrypted_response.encrypted_data.is_empty(), "Encrypted data should not be empty");
    assert_eq!(encrypted_response.nonce.len(), 12, "Nonce should be 12 bytes");

    // Decrypt on peer B's side
    let decrypted_secrets = vault_b
        .decrypt_from_peer(&encrypted_response)
        .expect("Failed to decrypt secrets");

    // Verify all secrets match
    assert_eq!(
        decrypted_secrets.get("DATABASE_URL"),
        Some(&"postgres://localhost/mydb".to_string()),
        "DATABASE_URL should match"
    );
    assert_eq!(
        decrypted_secrets.get("API_KEY"),
        Some(&"super-secret-key-12345".to_string()),
        "API_KEY should match"
    );
    assert_eq!(
        decrypted_secrets.get("DEBUG"),
        Some(&"true".to_string()),
        "DEBUG should match"
    );
    assert_eq!(decrypted_secrets.len(), 3, "Should have exactly 3 secrets");
}

#[test]
fn test_secrets_wrong_recipient_fails() {
    // Create three vaults
    let mut vault_a = EnvVault::new();
    let vault_b = EnvVault::new();
    let vault_c = EnvVault::new(); // Wrong recipient

    // Add secret to A
    vault_a.set("SECRET", "top-secret");

    // Encrypt for B
    let encrypted = vault_a
        .encrypt_for_peer(&vault_b.public_key())
        .expect("Failed to encrypt");

    // Try to decrypt with C (should fail)
    let result = vault_c.decrypt_from_peer(&encrypted);
    assert!(result.is_err(), "Decryption should fail for wrong recipient");
}

#[test]
fn test_env_file_parsing() {
    // Create a temporary .env file
    let temp_dir = std::env::temp_dir();
    let env_path = temp_dir.join("test_rift_secrets.env");

    let env_contents = r#"
# Database settings
DATABASE_URL=postgres://user:pass@localhost/db
DATABASE_POOL_SIZE=10

# API Configuration
API_KEY="quoted-key-value"
API_SECRET=unquoted_value

# Feature flags
ENABLE_CACHE=true
DEBUG=false
"#;

    std::fs::write(&env_path, env_contents).expect("Failed to write test .env file");

    // Load the file
    let vault = EnvVault::from_file(&env_path).expect("Failed to load .env file");

    // Verify parsed values
    assert_eq!(vault.get("DATABASE_URL"), Some(&"postgres://user:pass@localhost/db".to_string()));
    assert_eq!(vault.get("DATABASE_POOL_SIZE"), Some(&"10".to_string()));
    assert_eq!(vault.get("API_KEY"), Some(&"quoted-key-value".to_string()));
    assert_eq!(vault.get("API_SECRET"), Some(&"unquoted_value".to_string()));
    assert_eq!(vault.get("ENABLE_CACHE"), Some(&"true".to_string()));
    assert_eq!(vault.get("DEBUG"), Some(&"false".to_string()));

    // Cleanup
    let _ = std::fs::remove_file(env_path);
}

#[test]
fn test_empty_vault_encryption() {
    let vault_a = EnvVault::new();
    let vault_b = EnvVault::new();

    // Encrypt empty vault (should work)
    let encrypted = vault_a
        .encrypt_for_peer(&vault_b.public_key())
        .expect("Should encrypt empty vault");

    // Decrypt
    let decrypted = vault_b
        .decrypt_from_peer(&encrypted)
        .expect("Should decrypt empty vault");

    assert!(decrypted.is_empty(), "Decrypted vault should be empty");
}
