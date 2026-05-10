use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

fn derive_key(master_key: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let hk = Hkdf::<Sha256>::new(Some(salt), master_key.as_bytes());
    let mut okm = [0u8; KEY_LEN];
    hk.expand(b"HerbReady-db-config-v1", &mut okm)
        .expect("HKDF-Expand failed — buffer too short");
    okm
}

pub fn encrypt(plaintext: &str, master_key: &str) -> Result<String> {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let key = derive_key(master_key, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| anyhow!("Failed to create AES-256-GCM cipher"))?;

    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| anyhow!("Encryption failed"))?;

    let mut combined = salt.to_vec();
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

pub fn decrypt(encoded: &str, master_key: &str) -> Result<String> {
    let combined = BASE64.decode(encoded.as_bytes()).map_err(|e| anyhow!("Base64 decode error: {}", e))?;

    if combined.len() < SALT_LEN + NONCE_LEN {
        return Err(anyhow!("Invalid encrypted data — too short"));
    }

    let salt = &combined[..SALT_LEN];
    let nonce_bytes = &combined[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &combined[SALT_LEN + NONCE_LEN..];

    let key = derive_key(master_key, salt);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| anyhow!("Failed to create AES-256-GCM cipher"))?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow!("Decryption failed — wrong key or corrupted data"))?;

    String::from_utf8(plaintext).map_err(|e| anyhow!("UTF-8 decode error: {}", e))
}
