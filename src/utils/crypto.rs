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

/// Hash a password using bcrypt (stub — replace with `bcrypt` crate).
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    // TODO: use bcrypt::hash(password, bcrypt::DEFAULT_COST)
    Ok(format!("bcrypt_stub::{}", password))
}

/// Verify a password against its bcrypt hash (stub).
pub fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    // TODO: use bcrypt::verify(password, hash)
    Ok(hash == format!("bcrypt_stub::{}", password))
}
