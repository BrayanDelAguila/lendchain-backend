/// Cryptographic utilities for wallet key encryption and hashing.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::RngCore;

/// Encrypt a private key using AES-256-GCM.
/// The `encryption_key` must be at least 64 hex characters (32 bytes).
/// Stored format: base64(iv) + ":" + base64(ciphertext)
pub fn encrypt_private_key(private_key: &str, encryption_key: &str) -> anyhow::Result<String> {
    let key_bytes = hex_to_32_bytes(encryption_key)?;
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to build cipher: {}", e))?;

    let mut iv = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut iv);
    let nonce = Nonce::from_slice(&iv);

    let ciphertext = cipher
        .encrypt(nonce, private_key.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    Ok(format!("{}:{}", B64.encode(iv), B64.encode(ciphertext)))
}

/// Decrypt a private key encrypted with `encrypt_private_key`.
pub fn decrypt_private_key(encrypted: &str, encryption_key: &str) -> anyhow::Result<String> {
    let key_bytes = hex_to_32_bytes(encryption_key)?;
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to build cipher: {}", e))?;

    let (iv_b64, ct_b64) = encrypted
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("Invalid encrypted key format: missing ':'"))?;

    let iv = B64.decode(iv_b64)?;
    let ciphertext = B64.decode(ct_b64)?;
    let nonce = Nonce::from_slice(&iv);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(String::from_utf8(plaintext)?)
}

/// Derive 32 key bytes from the first 64 hex characters of `hex_key`.
fn hex_to_32_bytes(hex_key: &str) -> anyhow::Result<[u8; 32]> {
    if hex_key.len() < 64 {
        anyhow::bail!("WALLET_ENCRYPTION_KEY must be at least 64 hex characters (32 bytes)");
    }
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        bytes[i] = u8::from_str_radix(&hex_key[i * 2..i * 2 + 2], 16)
            .map_err(|_| anyhow::anyhow!("WALLET_ENCRYPTION_KEY contains non-hex characters"))?;
    }
    Ok(bytes)
}

/// Hash a password using Argon2id.
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Hash failed: {}", e))?;
    Ok(hash.to_string())
}

/// Verify a password against its Argon2 hash.
pub fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };
    let parsed =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Invalid hash: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = "my_super_secret_private_key_0xdeadbeef";
        let encrypted = encrypt_private_key(plaintext, TEST_KEY).expect("encrypt should succeed");
        let decrypted = decrypt_private_key(&encrypted, TEST_KEY).expect("decrypt should succeed");
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertexts() {
        let plaintext = "same_private_key";
        let enc1 = encrypt_private_key(plaintext, TEST_KEY).expect("first encrypt should succeed");
        let enc2 = encrypt_private_key(plaintext, TEST_KEY).expect("second encrypt should succeed");
        assert_ne!(enc1, enc2, "Each encryption should produce a unique IV and therefore a unique ciphertext");
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let plaintext = "secret_key_data";
        let wrong_key = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let encrypted = encrypt_private_key(plaintext, TEST_KEY).expect("encrypt should succeed");
        let result = decrypt_private_key(&encrypted, wrong_key);
        assert!(result.is_err(), "Decryption with wrong key should fail");
    }

    #[test]
    fn test_decrypt_invalid_format_fails() {
        let result = decrypt_private_key("no_colon_separator_here", TEST_KEY);
        assert!(result.is_err(), "Decryption of malformed input should fail");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("':'"), "Error message should mention the missing separator");
    }

    #[test]
    fn test_encrypt_short_key_fails() {
        let short_key = "0123456789abcdef"; // Only 16 hex chars, needs 64
        let result = encrypt_private_key("any_data", short_key);
        assert!(result.is_err(), "Encryption with key shorter than 64 hex chars should fail");
    }

    #[test]
    fn test_encrypt_empty_string() {
        let plaintext = "";
        let encrypted = encrypt_private_key(plaintext, TEST_KEY).expect("encrypt of empty string should succeed");
        let decrypted = decrypt_private_key(&encrypted, TEST_KEY).expect("decrypt of empty string should succeed");
        assert_eq!(plaintext, decrypted);
    }
}
