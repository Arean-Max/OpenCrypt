use crate::error::{CryptError, Result};
use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key};
use argon2::Argon2;
use base64::Engine;
use generic_array::GenericArray;
use zeroize::Zeroize;
use std::fs::File;
use std::io::{BufWriter, Read, Write};

pub const MAGIC: &[u8] = b"ORP\0";
pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const CHUNK_SIZE: usize = 1024 * 1024;

struct Header {
    salt: [u8; SALT_LEN],
    nonce: [u8; NONCE_LEN],
}

fn derive_key(password: &[u8], salt: &[u8]) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    let ctx = Argon2::default();
    ctx.hash_password_into(password, salt, &mut key)
        .map_err(|_| CryptError::Memory)?;
    Ok(key)
}

fn make_nonce(base: &[u8; NONCE_LEN], counter: u64) -> [u8; NONCE_LEN] {
    let mut n = *base;
    for i in 0..8 {
        n[NONCE_LEN - 8 + i] = ((counter >> (i * 8)) & 0xFF) as u8;
    }
    n
}

fn decode_key(b64: &str) -> Result<[u8; 32]> {
    let mut decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(b64)
        .map_err(|_| CryptError::InvalidParam)?;
    if decoded.len() != 32 {
        decoded.zeroize();
        return Err(CryptError::InvalidParam);
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded);
    decoded.zeroize();
    Ok(key)
}

fn read_header(file: &mut File) -> Result<Header> {
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    if &magic != MAGIC { return Err(CryptError::InvalidFormat); }
    let mut ver = [0u8; 1];
    file.read_exact(&mut ver)?;
    if ver[0] != 1 { return Err(CryptError::InvalidFormat); }
    let mut salt = [0u8; SALT_LEN];
    file.read_exact(&mut salt)?;
    let mut nonce = [0u8; NONCE_LEN];
    file.read_exact(&mut nonce)?;
    Ok(Header { salt, nonce })
}

fn write_header(writer: &mut BufWriter<File>, header: &Header) -> Result<()> {
    writer.write_all(MAGIC)?;
    writer.write_all(&[1])?;
    writer.write_all(&header.salt)?;
    writer.write_all(&header.nonce)?;
    Ok(())
}

fn read_exact_or_eof(file: &mut File, buf: &mut [u8]) -> Result<usize> {
    let mut offset = 0;
    while offset < buf.len() {
        match file.read(&mut buf[offset..]) {
            Ok(0) => break,
            Ok(n) => offset += n,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(offset)
}

fn encrypt_stream(
    reader: &mut File,
    writer: &mut BufWriter<File>,
    cipher: &Aes256Gcm,
    nonce_arr: &[u8; NONCE_LEN],
) -> Result<()> {
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut counter = 0u64;
    loop {
        let n = read_exact_or_eof(reader, &mut buf)?;
        if n == 0 { break; }
        let nce = make_nonce(nonce_arr, counter);
        let nce_ga = GenericArray::from_slice(&nce);
        let ct = cipher.encrypt(nce_ga, &buf[..n])
            .map_err(|_| CryptError::Memory)?;
        writer.write_all(&ct)?;
        counter += 1;
    }
    writer.flush()?;
    Ok(())
}

fn decrypt_stream(
    reader: &mut File,
    writer: &mut BufWriter<File>,
    cipher: &Aes256Gcm,
    header: &Header,
) -> Result<()> {
    let mut buf = vec![0u8; CHUNK_SIZE + 16];
    let mut counter = 0u64;
    loop {
        let n = read_exact_or_eof(reader, &mut buf)?;
        if n == 0 { break; }
        let nce = make_nonce(&header.nonce, counter);
        let nce_ga = GenericArray::from_slice(&nce);
        let pt = cipher.decrypt(nce_ga, &buf[..n])
            .map_err(|_| CryptError::InvalidPassword)?;
        writer.write_all(&pt)?;
        counter += 1;
    }
    writer.flush()?;
    Ok(())
}

pub fn encrypt_file(input: &str, output: &str, password: &str) -> Result<()> {
    let mut reader = File::open(input)?;
    let mut writer = BufWriter::new(File::create(output)?);

    let mut salt = [0u8; SALT_LEN];
    getrandom::getrandom(&mut salt).map_err(|_| CryptError::Memory)?;
    let mut key_bytes = derive_key(password.as_bytes(), &salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce_raw = Aes256Gcm::generate_nonce(&mut OsRng);
    let nonce_arr: [u8; NONCE_LEN] = nonce_raw.into();

    let header = Header { salt, nonce: nonce_arr };
    write_header(&mut writer, &header)?;
    encrypt_stream(&mut reader, &mut writer, &cipher, &nonce_arr)?;
    key_bytes.zeroize();
    Ok(())
}

pub fn decrypt_file(input: &str, output: &str, password: &str) -> Result<()> {
    let mut reader = File::open(input)?;
    let header = read_header(&mut reader)?;
    let mut key_bytes = derive_key(password.as_bytes(), &header.salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let result = decrypt_stream(&mut reader, &mut BufWriter::new(File::create(output)?), &cipher, &header);
    key_bytes.zeroize();
    result
}

pub fn encrypt_file_with_key(input: &str, output: &str, b64_key: &str) -> Result<()> {
    let mut raw_key = decode_key(b64_key)?;
    let cipher = {
        let key = Key::<Aes256Gcm>::from_slice(&raw_key);
        Aes256Gcm::new(key)
    };
    raw_key.zeroize();
    let nonce_raw = Aes256Gcm::generate_nonce(&mut OsRng);
    let nonce_arr: [u8; NONCE_LEN] = nonce_raw.into();

    let mut salt = [0u8; SALT_LEN];
    getrandom::getrandom(&mut salt).map_err(|_| CryptError::Memory)?;

    let mut reader = File::open(input)?;
    let mut writer = BufWriter::new(File::create(output)?);
    let header = Header { salt, nonce: nonce_arr };
    write_header(&mut writer, &header)?;
    encrypt_stream(&mut reader, &mut writer, &cipher, &nonce_arr)
}

pub fn decrypt_file_with_key(input: &str, output: &str, b64_key: &str) -> Result<()> {
    let mut raw_key = decode_key(b64_key)?;
    let cipher = {
        let key = Key::<Aes256Gcm>::from_slice(&raw_key);
        Aes256Gcm::new(key)
    };
    raw_key.zeroize();
    let mut reader = File::open(input)?;
    let header = read_header(&mut reader)?;
    decrypt_stream(&mut reader, &mut BufWriter::new(File::create(output)?), &cipher, &header)
}

pub fn verify_password(input: &str, password: &str) -> Result<bool> {
    let mut reader = File::open(input)?;
    let header = read_header(&mut reader)?;
    let mut key_bytes = derive_key(password.as_bytes(), &header.salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let mut buf = vec![0u8; CHUNK_SIZE + 16];
    let n = read_exact_or_eof(&mut reader, &mut buf)?;
    if n == 0 { return Ok(true); }
    let nce = make_nonce(&header.nonce, 0);
    let nce_ga = GenericArray::from_slice(&nce);
    let ok = cipher.decrypt(nce_ga, &buf[..n]).is_ok();
    key_bytes.zeroize();
    Ok(ok)
}

pub fn secure_delete(path: &str) -> Result<()> {
    let len = std::fs::metadata(path)?.len();
    if len > 0 {
        let mut f = std::fs::OpenOptions::new().write(true).open(path)?;
        let block = 65536usize;
        let mut buf = vec![0u8; block];
        getrandom::getrandom(&mut buf).map_err(|_| CryptError::Memory)?;
        let mut remaining = len;
        while remaining > 0 {
            let n = remaining.min(block as u64) as usize;
            f.write_all(&buf[..n])?;
            remaining -= n as u64;
        }
        f.flush()?;
    }
    std::fs::remove_file(path)?;
    Ok(())
}

pub fn generate_random_key() -> String {
    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key).expect("OS RNG failed");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&key)
}
