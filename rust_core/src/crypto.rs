use crate::error::{CryptError, Result};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::Engine;

pub const MAGIC: &[u8] = b"ORP\0";
pub const FORMAT_VERSION: u8 = 1;
pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const CHUNK_SIZE: usize = 1024 * 1024;

pub const HEADER_LEN: usize = 41;

pub fn generate_random_key() -> Result<String> {
    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key).map_err(|_| CryptError::Memory)?;
    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&key);
    Ok(b64)
}

fn decode_key(b64: &str) -> Result<[u8; 32]> {
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(b64)
        .map_err(|_| CryptError::InvalidParam)?;
    if decoded.len() != 32 {
        return Err(CryptError::InvalidParam);
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded);
    Ok(key)
}

fn derive_key(password: &[u8], salt: &[u8]) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    let ctx = argon2::Argon2::default();
    ctx.hash_password_into(password, salt, &mut key)
        .map_err(|_| CryptError::Memory)?;
    Ok(key)
}

fn validate_file_length(len: u64) -> Result<()> {
    if (len as usize) < HEADER_LEN {
        return Err(CryptError::InvalidFormat);
    }
    Ok(())
}
