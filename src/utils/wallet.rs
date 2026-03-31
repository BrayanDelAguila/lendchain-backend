/// Custodial wallet generation using ethers-rs.
use ethers::signers::{LocalWallet, Signer};

use crate::utils::crypto::encrypt_private_key;

/// A custodial wallet: public address + AES-256-GCM encrypted private key.
pub struct CustodialWallet {
    /// Public Ethereum address in checksummed hex format: `0xABCD...`
    pub address: String,
    /// AES-256-GCM ciphertext of the raw private key (base64(iv):base64(ct)).
    pub encrypted_private_key: String,
}

/// Generate a new random custodial wallet and encrypt its private key.
///
/// `encryption_key` must be at least 64 hex characters (32 bytes).
pub fn generate_custodial_wallet(encryption_key: &str) -> anyhow::Result<CustodialWallet> {
    let wallet = LocalWallet::new(&mut rand::thread_rng());

    let address = ethers::utils::to_checksum(&wallet.address(), None);

    // Extract raw private key bytes and encode as lowercase hex.
    let pk_bytes = wallet.signer().to_bytes();
    let private_key_hex: String = pk_bytes.iter().map(|b| format!("{:02x}", b)).collect();

    let encrypted_private_key = encrypt_private_key(&private_key_hex, encryption_key)?;

    Ok(CustodialWallet {
        address,
        encrypted_private_key,
    })
}
