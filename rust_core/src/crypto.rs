use crate::error::{CryptError, Result};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::Engine;

pub fn generate_random_key() -> Result<String> {
    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key).map_err(|_| CryptError::Memory)?;
    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&key);
    Ok(b64)
}

pub fn encrypt_buffer(plaintext: &[u8], key_bytes: &[u8; 32], nonce_bytes: &[u8; 12]) -> Result<Vec<u8>> {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.encrypt(nonce, plaintext).map_err(|_| CryptError::InvalidPassword)
}

pub fn decrypt_buffer(ciphertext: &[u8], key_bytes: &[u8; 32], nonce_bytes: &[u8; 12]) -> Result<Vec<u8>> {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext).map_err(|_| CryptError::InvalidPassword)
}
