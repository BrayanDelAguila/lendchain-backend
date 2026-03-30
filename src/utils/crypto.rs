/// Cryptographic utilities for wallet key encryption and hashing.

/// Encrypt a private key using AES-256-GCM (stub — replace with real implementation).
/// In production use a crate like `aes-gcm` with the WALLET_ENCRYPTION_KEY.
pub fn encrypt_private_key(private_key: &str, _encryption_key: &str) -> anyhow::Result<String> {
    // TODO: implement AES-256-GCM encryption
    // This is a stub that base64-encodes the key for development only.
    use std::fmt::Write;
    let mut hex = String::new();
    for b in private_key.as_bytes() {
        let _ = write!(hex, "{:02x}", b);
    }
    Ok(format!("stub::{}", hex))
}

/// Decrypt a previously encrypted private key (stub).
pub fn decrypt_private_key(encrypted: &str, _encryption_key: &str) -> anyhow::Result<String> {
    // TODO: implement AES-256-GCM decryption
    let hex_part = encrypted
        .strip_prefix("stub::")
        .ok_or_else(|| anyhow::anyhow!("Invalid encrypted key format"))?;
    let bytes = (0..hex_part.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_part[i..i + 2], 16))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(String::from_utf8(bytes)?)
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
